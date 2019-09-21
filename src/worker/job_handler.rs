use std::sync::Arc;
use std::pin::Pin;
use futures::{Future, FutureExt, TryFuture, TryFutureExt};
use std::panic::AssertUnwindSafe;

pub struct JobHandler {
    job_handler: Arc<dyn Fn(crate::ActivatedJob) -> Pin<Box<dyn Future<Output = crate::JobResult> + Send>> + Send + Sync>,
}

impl JobHandler {
    pub fn process_job(&self, activated_job: crate::ActivatedJob) -> impl Future<Output = Result<crate::JobResult, ()>> {

        AssertUnwindSafe((self.job_handler)(activated_job)).catch_unwind().map_err(|_| ())
    }

    pub fn new(job_handler:  Arc<dyn Fn(crate::ActivatedJob) -> Pin<Box<dyn Future<Output = crate::JobResult> + Send>> + Send + Sync>) -> Self {
        Self {
            job_handler,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{JobHandler, ActivatedJob};
    use crate::JobResult;
    use std::sync::Arc;
    use futures::FutureExt;

    #[test]
    fn catches_on_panic() {
        let jh = JobHandler::new(Arc::new(|_| {
            panic!("oh no!");
        }));
        let aj = ActivatedJob {
            key: 0,
            field_type: "".to_string(),
            custom_headers: "".to_string(),
            worker: "".to_string(),
            retries: 0,
            deadline: 0,
            variables: "".to_string(),
        };
        jh.process_job(aj);
    }
}