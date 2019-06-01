use crate::activate_jobs::activate_jobs;
use crate::complete_job::complete_job;
use crate::fail_job::fail_job;
use crate::{gateway, gateway_grpc, ActivateJobsConfig, CompletedJobData, Error, ActivatedJob};
use futures::future::Future;
use futures::{IntoFuture, Stream};
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
    f: Arc<F>,
) -> impl Stream<Item = CompletedJobData, Error = Error>
where
    F: Fn(gateway::ActivatedJob) -> X + Send,
    X: IntoFuture<Item = Option<String>, Error = JobError>,
{
    futures::stream::empty()
//    let client = gateway_client.clone();
//    let client_ref = gateway_client.as_ref();
//    activate_jobs(client_ref, &worker_config.activate_jobs_config)
//        .map_err(|e| Error::ActivateJobError(e))
//        .zip(futures::stream::repeat((f, client.clone())))
//        .and_then(|(activated_job, (f, client))| {
//            let client2 = client.clone();
//            futures::future::ok(activated_job.clone())
//                .join((f)(activated_job.clone()))
//                .map_err(move |job_error| {
//                    let retries = match job_error {
//                        JobError::Retry { .. } => {
//                            let mut retries = activated_job.retries - 1;
//                            if retries <= 0 {
//                                retries = 0;
//                            }
//                            retries
//                        }
//                        JobError::Fail { .. } => 0,
//                    };
//                    // TODO: put this spawn back into the future control flow so that the result is captured
//                    tokio::spawn(
//                        fail_job(client.as_ref(), activated_job.key, retries).map_err(|_| ()),
//                    );
//                    Error::JobError(job_error)
//                })
//                .join(futures::future::ok(client2))
//                .and_then(|((activated_job, payload), client)| {
//                    let completed_job_data = CompletedJobData {
//                        job_key: activated_job.key,
//                        payload,
//                    };
//                    complete_job(&client, completed_job_data)
//                        .map_err(|e| Error::CompleteJobError(e))
//                })
//        })
}

pub(crate) fn activate_and_process_jobs_2<F, X>(
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
    let client_ref = gateway_client.as_ref();
    activate_jobs(client_ref, &worker_config.activate_jobs_config)
        .map_err(|e| Error::ActivateJobError(e))
        .and_then(move |activated_job| {
            let activated_job_cloned = activated_job.clone();
            job_fn.call(activated_job.clone()).then(|result| {
                let result: Result<JobResponse, JobError> = result;
                match result {
                    Ok(response) => Ok(response),
                    Err(error) => Ok(error.into()),
                }
            }).and_then(|response| {
                on_job_finished(response, activated_job.clone(), client.clone())
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

/*
fn on_job_finished(job_result: Result<(Option<String>, ActivatedJob, Arc<GatewayClient>), (JobError, ActivatedJob, Arc<GatewayClient>)>) -> Box<dyn Future<Item = Result<CompletedJobData, JobError>, Error = Error>> {
    match job_result {
        Ok((payload, activated_job, client)) => {
            let completed_job_data = CompletedJobData {
                job_key: activated_job.key,
                payload,
            };
            let f = complete_job(&client, completed_job_data)
                .map_err(|e| Error::CompleteJobError(e));
            Box::new(f)
        },
        Err((job_error, activated_job, client)) => {
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

            let f = fail_job(client.as_ref(), activated_job.key, retries)
                .map_err(|e| Error::FailJobError(e))
                .map(|_| futures::future::ok(job_error))
            Error::JobError(job_error)
        },
    }
}*/
