use crate::{ActivatedJob, Client, CompleteJob, JobResult};
use futures::{Future, FutureExt};
use std::pin::Pin;
use std::sync::Arc;

pub trait JobStatusReporter {
    fn complete(
        &self,
        key: i64,
        variables: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<(), crate::Error>> + Send>>;
    fn fail(
        &self,
        key: i64,
        retries: i32,
        error_message: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<(), crate::Error>> + Send>>;
}

pub struct Reporter {
    client: Client,
}

impl Reporter {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

impl JobStatusReporter for Reporter {
    fn complete(
        &self,
        key: i64,
        variables: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<(), crate::Error>> + Send>> {
        let complete_job = CompleteJob {
            job_key: key,
            variables,
        };
        self.client.complete_job(complete_job).boxed()
    }

    fn fail(
        &self,
        key: i64,
        retries: i32,
        error_message: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<(), crate::Error>> + Send>> {
        let error_message = error_message.unwrap_or("".to_string());
        self.client
            .fail_job(key, retries - 1, error_message)
            .boxed()
    }
}

pub struct JobClient {
    complete: Arc<dyn JobStatusReporter + Send + Sync>,
}

impl JobClient {
    pub fn new<T: JobStatusReporter + Send + Sync + 'static>(completer: T) -> Self {
        Self {
            complete: Arc::new(completer),
        }
    }

    pub fn report_status(
        &self,
        activated_job: ActivatedJob,
        job_result: JobResult,
    ) -> Pin<Box<dyn Future<Output = Result<(), crate::Error>> + Send>> {
        let key = activated_job.key;
        let retries = activated_job.retries;
        match job_result {
            JobResult::NoAction => futures::future::ok(()).boxed(),
            JobResult::Fail { error_message } => match error_message {
                Some(msg) => self.complete.fail(key, retries, Some(msg)),
                None => self.complete.fail(key, retries, None),
            },
            JobResult::Complete { variables } => self.complete.complete(key, variables),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::worker::job_client::JobStatusReporter;
    use crate::{JobClient, JobResult};
    use futures::{Future, FutureExt};
    use std::pin::Pin;
    use std::sync::{Arc, RwLock};

    #[derive(Clone)]
    struct MockReporter {
        completed: Arc<RwLock<Option<i64>>>,
        fail: Arc<RwLock<Option<i64>>>,
    }

    impl MockReporter {
        pub fn new() -> Self {
            Self {
                completed: Arc::new(RwLock::new(None)),
                fail: Arc::new(RwLock::new(None)),
            }
        }
    }

    impl JobStatusReporter for MockReporter {
        fn complete(
            &self,
            key: i64,
            _variables: Option<String>,
        ) -> Pin<Box<dyn Future<Output = Result<(), crate::Error>> + Send>> {
            let mut write = self.completed.write().unwrap();
            *write = Some(key);
            futures::future::ok(()).boxed()
        }

        fn fail(
            &self,
            key: i64,
            _retries: i32,
            _error_message: Option<String>,
        ) -> Pin<Box<dyn Future<Output = Result<(), crate::Error>> + Send>> {
            let mut write = self.fail.write().unwrap();
            *write = Some(key);
            futures::future::ok(()).boxed()
        }
    }

    #[test]
    fn calls_client_method_on_completed_job() {
        let completer = MockReporter::new();
        let job_client = JobClient {
            complete: Arc::new(completer.clone()),
        };
        let activated_job = crate::ActivatedJob {
            key: 100,
            field_type: "".to_string(),
            custom_headers: "".to_string(),
            worker: "".to_string(),
            retries: 0,
            deadline: 0,
            variables: "".to_string(),
        };
        let result = JobResult::Complete { variables: None };
        let _ =
            futures::executor::block_on(job_client.report_status(activated_job, result)).unwrap();
        let completed = completer.completed.read().unwrap().clone();
        let failed = completer.fail.read().unwrap().clone();
        assert_eq!(completed, Some(100));
        assert_eq!(failed, None);
    }

    #[test]
    fn calls_client_method_on_failed_job() {
        let completer = MockReporter::new();
        let job_client = JobClient {
            complete: Arc::new(completer.clone()),
        };
        let activated_job = crate::ActivatedJob {
            key: 100,
            field_type: "".to_string(),
            custom_headers: "".to_string(),
            worker: "".to_string(),
            retries: 0,
            deadline: 0,
            variables: "".to_string(),
        };
        let result = JobResult::Fail {
            error_message: None,
        };
        let _ =
            futures::executor::block_on(job_client.report_status(activated_job, result)).unwrap();
        let completed = completer.completed.read().unwrap().clone();
        let failed = completer.fail.read().unwrap().clone();
        assert_eq!(completed, None);
        assert_eq!(failed, Some(100));
    }
}
