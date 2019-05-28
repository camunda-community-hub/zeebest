use crate::activate_jobs::{activate_jobs, ActivateJobsConfig};
use crate::client::Error;
use crate::complete_job::{complete_job, CompletedJobData};
use crate::{gateway, gateway_grpc};
use futures::future::Future;
use futures::{IntoFuture, Stream};
use std::sync::Arc;

#[derive(Clone)]
pub struct WorkerConfig {
    pub activate_jobs_config: ActivateJobsConfig,
    pub cancel_workflow_on_panic: bool,
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
        .zip(futures::stream::repeat(f))
        .and_then(|(y, f)| {
            futures::future::ok(y.key)
                .join((f)(y))
                .map_err(|e| Error::JobError(e))
        })
        .zip(futures::stream::repeat(client))
        .and_then(|((job_key, payload), client)| {
            let completed_job_data = CompletedJobData { job_key, payload };
            complete_job(&client, completed_job_data).map_err(|e| Error::CompleteJobError(e))
        })
}
