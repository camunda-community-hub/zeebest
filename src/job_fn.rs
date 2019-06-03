use crate::activate_and_process_jobs::{JobResponse, FutureJobResponse};
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
        F: Fn(gateway::ActivatedJob) -> X + Send + 'static,
        X: IntoFuture<Item = JobResponse, Error = JobError> + 'static,
        <X as futures::future::IntoFuture>::Future: std::panic::UnwindSafe + 'static,
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

pub trait JobFnLike {
    fn job_type(&self) -> String;
    fn timeout(&self) -> Option<i64>;
    fn amount(&self) -> Option<i32>;
    fn panic_option(&self) -> Option<PanicOption>;
    fn call(
        &self,
        activated_job: ActivatedJob,
    ) -> FutureJobResponse;
}

impl<F,X> JobFnLike for JobFn<F,X>
    where
        F: Fn(gateway::ActivatedJob) -> X + Send + 'static,
        X: IntoFuture<Item = JobResponse, Error = JobError> + 'static,
        <X as futures::future::IntoFuture>::Future: std::panic::UnwindSafe + 'static,
{
    fn job_type(&self) -> String {
        self.job_type.clone()
    }

    fn timeout(&self) -> Option<i64> {
        self.timeout.clone()
    }

    fn amount(&self) -> Option<i32> {
        self.amount.clone()
    }

    fn panic_option(&self) -> Option<PanicOption> {
        self.panic_option
    }

    fn call(&self, activated_job: ActivatedJob) -> FutureJobResponse {
        let job_fn_result = self.call(activated_job);
        FutureJobResponse::from_future(job_fn_result)
    }
}

pub fn handle_panic(
    f: FutureJobResponse,
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
