use futures::IntoFuture;

pub struct JobWorker<F, H>
    where
        F: IntoFuture<Item = Option<String>, Error = ()>,
        H: Fn(i64, String) -> F,
{
    name: String,
    job_type: String,
    max_running_jobs: Option<usize>,
    handler: H,
}

impl<F, H> JobWorker<F, H>
    where
        F: IntoFuture<Item = Option<String>, Error = ()>,
        H: Fn(i64, String) -> F,
{
    pub fn new(name: String, job_type: String, handler: H) -> Self {
        JobWorker {
            name,
            job_type,
            max_running_jobs: None,
            handler,
        }
    }

    pub fn call_handler(&self, job_key: i64, payload: String) -> F::Future {
        (self.handler)(job_key, payload).into_future()
    }
}

#[cfg(test)]
mod test {
    use crate::worker::job_worker::JobWorker;

    #[test]
    fn make_worker() {
        let worker = JobWorker::new(
            "foo-worker".to_string(),
            "payment-process".to_string(),
            |_key, _payload| Ok(None),
        );

        let _f = worker.call_handler(1, "payload".to_string());
    }
}
