use crate::activate_jobs::ActivateJobsConfig;

use futures::{Future, IntoFuture};

#[derive(Clone)]
pub struct WorkerConfig {
    pub activate_jobs_config: ActivateJobsConfig,
    pub cancel_workflow_on_panic: bool,
}

pub struct WorkerFn<F, X>
where
    F: Fn(i64, String) -> X + Send + 'static,
    X: IntoFuture<Item = Option<String>, Error = ()> + Send,
    <X as IntoFuture>::Future: Send,
{
    f: F,
}

impl<F, X> WorkerFn<F, X>
where
    F: Fn(i64, String) -> X + Send + 'static,
    X: IntoFuture<Item = Option<String>, Error = ()> + Send,
    <X as IntoFuture>::Future: Send,
{
    pub fn new(f: F) -> Self {
        WorkerFn { f }
    }

    pub fn call(
        &self,
        job_key: i64,
        payload: String,
    ) -> impl Future<Item = (i64, Option<String>), Error = ()> + Send {
        futures::future::ok::<i64, ()>(job_key).join((self.f)(job_key, payload))
    }
}
