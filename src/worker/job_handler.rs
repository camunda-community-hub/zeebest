use std::sync::Arc;
use std::pin::Pin;
use futures::{Future, FutureExt, TryFuture, TryFutureExt, TryStreamExt, StreamExt};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::panic;

pub struct JobHandler {
    job_handler: Arc<dyn Fn(crate::ActivatedJob) -> Pin<Box<dyn Future<Output = crate::JobResult> + Send>> + Send + Sync>,
}

impl JobHandler {
    pub fn process_job(&self, activated_job: crate::ActivatedJob) -> Pin<Box<dyn Future<Output = Result<crate::JobResult, ()>> + Send>> {
        let job_handler = self.job_handler.clone();
        let result = panic::catch_unwind(AssertUnwindSafe(|| (job_handler)(activated_job)));
        match result {
            Err(_) => futures::future::err(()).boxed(),
            Ok(f) => {
                AssertUnwindSafe(f).catch_unwind().then(|r| {
                    match r {
                        Err(_) => futures::future::err(()),
                        Ok(jr) => futures::future::ok(jr),
                    }
                }).boxed()
            }
        }
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
        let result = futures::executor::block_on(jh.process_job(aj));
        assert!(result.is_err(), "Job panicked but did not error");
    }

    #[test]
    fn returns_ok_result() {
        let jh = JobHandler::new(Arc::new(|_| {
            futures::future::ready(JobResult::NoAction).boxed()
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
        let result = futures::executor::block_on(jh.process_job(aj));
        assert!(result.is_ok(), "Job errored with non-panicking handler");
    }
}