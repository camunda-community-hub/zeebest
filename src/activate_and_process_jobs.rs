use crate::activate_jobs::activate_jobs;
use crate::complete_job::complete_job;
use crate::fail_job::fail_job;
use crate::{gateway, gateway_grpc, ActivateJobsConfig, CompletedJobData, Error, ActivatedJob};
use futures::future::Future;
use futures::{IntoFuture, Stream, Async};
use std::sync::Arc;
use crate::gateway_grpc::GatewayClient;
use crate::job_fn::JobFn;

#[derive(Clone)]
pub struct WorkerConfig {
    pub activate_jobs_config: ActivateJobsConfig,
    pub cancel_workflow_on_panic: bool, // TODO: make this do something
}

#[derive(Debug)]
pub struct JobError { reason: Option<String>, retry: bool, }

#[derive(Clone, Debug)]
pub enum JobResponse {
    Complete { payload: Option<String> },
    Retry { reason: Option<String> },
    Fail { reason: Option<String> },
    DoNothing,
}

impl IntoFuture for JobResponse {
    type Future = FutureJobResponse;
    type Item = JobResponse;
    type Error = JobError;

    fn into_future(self) -> Self::Future {
        FutureJobResponse::new(self)
    }
}

pub struct FutureJobResponse {
    inner: Option<JobResponse>,
}

impl FutureJobResponse {
    pub fn new(job_response: JobResponse) -> Self {
        FutureJobResponse { inner: Some(job_response) }
    }
}

impl Future for FutureJobResponse {
    type Item = JobResponse;
    type Error = JobError;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        Ok(Async::Ready(self.inner.take().expect("cannot poll same job twice")))
    }
}

impl Into<JobResponse> for JobError {
    fn into(self) -> JobResponse {
        if self.retry {
            JobResponse::Retry { reason: self.reason }
        }
        else {
            JobResponse::Fail { reason: self.reason }
        }
    }
}

#[derive(Debug)]
pub struct JobResult {
    job: ActivatedJob,
    command: JobResponse,
}

impl JobResult {
    pub fn new(job: ActivatedJob,
               command: JobResponse,) -> Self {
        JobResult {
            job, command,
        }
    }
}

pub(crate) fn activate_and_process_jobs<F, X>(
    gateway_client: Arc<gateway_grpc::GatewayClient>,
    worker_config: WorkerConfig,
    job_fn: JobFn<F,X>,
) -> impl Stream<Item = JobResult, Error = Error>
    where
        F: Fn(gateway::ActivatedJob) -> X + Send,
        X: IntoFuture<Item = JobResponse, Error = JobError>,
        <X as futures::future::IntoFuture>::Future: std::panic::UnwindSafe,
{
    let client = gateway_client.clone();
    let client_ref = client.as_ref();
    activate_jobs(client_ref, &worker_config.activate_jobs_config)
        .map_err(|e| Error::ActivateJobError(e))
        .and_then(move |activated_job| {
            let activated_job_cloned = activated_job.clone();
            let client_cloned = gateway_client.clone();
            job_fn.call(activated_job.clone()).then(move |result| {
                let result: Result<JobResponse, JobError> = result;
                match result {
                    Ok(response) => Ok(response),
                    Err(error) => Ok(error.into()),
                }
            }).join(futures::future::ok(client_cloned)).and_then(|(response, client)| {
                on_job_finished(response, activated_job_cloned, client)
            })
        })
}

//JobResult
fn on_job_finished(job_response: JobResponse, activated_job: ActivatedJob, client: Arc<GatewayClient>) -> Box<dyn Future<Item = JobResult, Error = Error>> {
    match job_response.clone() {
        JobResponse::Complete { payload } => {
            let completed_job_data = CompletedJobData {
                job_key: activated_job.key,
                payload,
            };
            let f = complete_job(&client, completed_job_data)
                .map(move |_| JobResult::new(activated_job, job_response, ))
                .map_err(|e| Error::CompleteJobError(e));
            Box::new(f)
        },
        JobResponse::Retry { ..} => {
            let mut retries = activated_job.retries - 1;
            if retries <= 0 {
                retries = 0;
            }
            let f = fail_job(client.as_ref(), activated_job.key, retries)
                .map(move |_| JobResult::new(activated_job, job_response, ));
            Box::new(f)
        },
        JobResponse::Fail { ..} => {
            let f = fail_job(client.as_ref(), activated_job.key, 0)
                .map(move |_| JobResult::new(activated_job, job_response, ));
            Box::new(f)
        },
        JobResponse::DoNothing => {
            Box::new(futures::future::ok(JobResult::new(activated_job, job_response, )))
        }
    }
}
