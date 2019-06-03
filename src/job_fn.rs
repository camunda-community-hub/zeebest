use crate::activate_and_process_jobs::JobResponse;
use crate::{gateway, ActivatedJob, JobError};
use futures::{Future, IntoFuture};
use std::sync::Arc;

#[derive(Clone, Copy)]
pub enum PanicOption {
    FailJobOnPanic,
    DoNothingOnPanic,
}

pub struct JobFn<F, X>
where
    F: Fn(gateway::ActivatedJob) -> X + Send,
    X: IntoFuture<Item = JobResponse, Error = JobError>,
    <X as futures::future::IntoFuture>::Future: std::panic::UnwindSafe,
{
    pub(crate) job_type: String,
    f: Arc<F>,
    pub(crate) panic_option: Option<PanicOption>,
    pub(crate) timeout: Option<i64>,
    pub(crate) amount: Option<i32>,
}

impl<F, X> Clone for JobFn<F, X>
where
    F: Fn(gateway::ActivatedJob) -> X + Send,
    X: IntoFuture<Item = JobResponse, Error = JobError>,
    <X as futures::future::IntoFuture>::Future: std::panic::UnwindSafe,
{
    fn clone(&self) -> Self {
        JobFn {
            f: self.f.clone(),
            panic_option: self.panic_option.clone(),
            amount: self.amount.clone(),
            timeout: self.timeout.clone(),
            job_type: self.job_type.clone(),
        }
    }
}

impl<F, X> JobFn<F, X>
where
    F: Fn(gateway::ActivatedJob) -> X + Send,
    X: IntoFuture<Item = JobResponse, Error = JobError>,
    <X as futures::future::IntoFuture>::Future: std::panic::UnwindSafe,
{
    pub fn call(
        &self,
        activated_job: ActivatedJob,
    ) -> impl Future<Item = JobResponse, Error = JobError> {
        ((self.f)(activated_job)).into_future()
    }

    pub fn with_job_type_and_handler<S: Into<String>>(job_type: S, f: F) -> Self {
        JobFn {
            job_type: job_type.into(),
            f: Arc::new(f),
            panic_option: None,
            timeout: None,
            amount: None,
        }
    }

    pub fn panic_option(mut self, panic_option: PanicOption) -> Self {
        self.panic_option = Some(panic_option);
        self
    }

    pub fn timeout(mut self, timeout: i64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn amount(mut self, amount: i32) -> Self {
        self.amount = Some(amount);
        self
    }
}

pub fn handle_panic<F: Future<Item = JobResponse, Error = JobError> + std::panic::UnwindSafe>(
    f: F,
activated_job_key: i64,
            panic_option: PanicOption,
) -> impl Future<Item = JobResponse, Error = JobError> {
    f.catch_unwind()
        .then(move |r: Result<Result<JobResponse, JobError>, _>| match r {
            Ok(job_result) => job_result,
            Err(_panic_error) => match panic_option {
                PanicOption::FailJobOnPanic => Ok(JobResponse::Retry {
                    reason: Some(format!("Job #{} panicked.", activated_job_key)),
                }),
                PanicOption::DoNothingOnPanic => Ok(JobResponse::DoNothing),
            },
        })
        .then(move |result| {
            let result: Result<JobResponse, JobError> = result;
            match result {
                Ok(response) => Ok(response),
                Err(error) => Ok(error.into()),
            }
        })
}
