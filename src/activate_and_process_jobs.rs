use crate::activate_jobs::activate_jobs;
use crate::complete_job::complete_job;
use crate::fail_job::fail_job;
use crate::{gateway, gateway_grpc, ActivateJobsConfig, CompletedJobData, Error};
use futures::future::Future;
use futures::{IntoFuture, Stream};
use std::sync::Arc;

#[derive(Clone)]
pub struct WorkerConfig {
    pub activate_jobs_config: ActivateJobsConfig,
    pub cancel_workflow_on_panic: bool, // TODO: make this do something
}

#[derive(Debug)]
pub enum JobError {
    Retry { msg: String },
    Fail { msg: String },
}

pub(crate) fn activate_and_process_jobs<F, X>(
    gateway_client: Arc<gateway_grpc::GatewayClient>,
    worker_config: WorkerConfig,
    f: Arc<F>,
) -> impl Stream<Item = CompletedJobData, Error = Error>
where
    F: Fn(gateway::ActivatedJob) -> X + Send,
    X: IntoFuture<Item = Option<String>, Error = JobError>,
{
    let client = gateway_client.clone();
    let client_ref = gateway_client.as_ref();
    activate_jobs(client_ref, &worker_config.activate_jobs_config)
        .map_err(|e| Error::ActivateJobError(e))
        .zip(futures::stream::repeat((f, client.clone())))
        .and_then(|(activated_job, (f, client))| {
            let client2 = client.clone();
            futures::future::ok(activated_job.clone())
                .join((f)(activated_job.clone()))
                .map_err(move |job_error| {
                    let retries = match job_error {
                        JobError::Retry { .. } => {
                            let mut retries = activated_job.retries - 1;
                            if retries <= 0 {
                                retries = 0;
                            }
                            retries
                        }
                        JobError::Fail { .. } => 0,
                    };
                    // TODO: put this spawn back into the future control flow so that the result is captured
                    tokio::spawn(
                        fail_job(client.as_ref(), activated_job.key, retries).map_err(|_| ()),
                    );
                    Error::JobError(job_error)
                })
                .join(futures::future::ok(client2))
                .and_then(|((activated_job, payload), client)| {
                    let completed_job_data = CompletedJobData {
                        job_key: activated_job.key,
                        payload,
                    };
                    complete_job(&client, completed_job_data)
                        .map_err(|e| Error::CompleteJobError(e))
                })
        })
}
