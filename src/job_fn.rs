use futures::{Future, IntoFuture};
use crate::activate_and_process_jobs::JobResponse;
use crate::{JobError, gateway, ActivatedJob};
use std::sync::Arc;

#[derive(Clone, Copy)]
pub enum PanicOption {
    FailJobOnPanic,
    DoNothingOnPanic,
}

pub struct JobFn<F, X> where
    F: Fn(gateway::ActivatedJob) -> X + Send,
    X: IntoFuture<Item = JobResponse, Error = JobError>,
    <X as futures::future::IntoFuture>::Future: std::panic::UnwindSafe,
{
    f: Arc<F>,
    panic_option: PanicOption,
}

impl<F,X> Clone for JobFn<F, X> where
    F: Fn(gateway::ActivatedJob) -> X + Send,
    X: IntoFuture<Item = JobResponse, Error = JobError>,
    <X as futures::future::IntoFuture>::Future: std::panic::UnwindSafe, {
    fn clone(&self) -> Self {
        JobFn { f: self.f.clone(), panic_option: self.panic_option.clone() }
    }
}

impl<F,X> JobFn<F,X> where
    F: Fn(gateway::ActivatedJob) -> X + Send,
    X: IntoFuture<Item = JobResponse, Error = JobError>,
    <X as futures::future::IntoFuture>::Future: std::panic::UnwindSafe,
{
    pub fn call(&self, activated_job: ActivatedJob) -> impl Future<Item = JobResponse, Error = JobError> {
        let panic_option = self.panic_option;
        let activated_job_key = activated_job.get_key();
        ((self.f)(activated_job)).into_future().catch_unwind().then(move |r: Result<Result<JobResponse, JobError>, _>| {
            match r {
                Ok(job_result) => {
                    job_result
                },
                Err(_panic_error) => {
                    match panic_option {
                        PanicOption::FailJobOnPanic => {
                            Ok(JobResponse::Retry { reason: Some(format!("Job #{} panicked.", activated_job_key)) })
                        },
                        PanicOption::DoNothingOnPanic => {
                            Ok(JobResponse::DoNothing)
                        },
                    }
                }
            }
        })
    }

    pub fn new(f: F, panic_option: PanicOption) -> Self {
        JobFn {
            f:  Arc::new(f),
            panic_option,
        }
    }
}
