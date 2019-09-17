use crate::{ActivateJobs, ActivatedJob, ActivatedJobs, Client, CompleteJob};
use futures::{Future, FutureExt, StreamExt};
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// An option that describes what the job worker should do if if the job handler panics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PanicOption {
    FailJobOnPanic,
    DoNothingOnPanic,
}

/// A result that describes the output of a job.
#[derive(Clone, Debug, PartialEq)]
pub enum JobResult {
    Complete { variables: Option<String> },
    Fail { error_message: Option<String> },
    NoAction,
}

impl JobResult {
    pub fn into_result(self) -> Result<Option<String>, Option<String>> {
        match self {
            JobResult::Complete { variables } => Ok(variables),
            JobResult::Fail { error_message } => Err(error_message),
            JobResult::NoAction => Err(None),
        }
    }
}

type JobHandlerFn =
    Box<dyn Fn(ActivatedJob) -> Pin<Box<dyn Future<Output = JobResult> + Send>> + Send + Sync>;

pub struct JobInternal {
    job_handler: JobHandlerFn,
    job_count: AtomicUsize,
    max_concurrent_jobs: usize,
    client: Client,
    worker_name: String,
    job_type: String,
    timeout: i64,
    panic_option: PanicOption,
}

impl JobInternal {
    pub async fn activate_and_process_jobs(self: Arc<Self>) {
        let current_job_count: usize = self.job_count.load(Ordering::SeqCst);
        // assert on an unreachable edge case
        assert!(
            current_job_count <= self.max_concurrent_jobs,
            "current number of jobs exceeds allowed number of running jobs"
        );
        let mut activate_jobs = ActivateJobs::new(
            self.worker_name.clone(),
            self.job_type.clone(),
            self.timeout,
            0,
        );
        activate_jobs.max_jobs_to_activate = (self.max_concurrent_jobs - current_job_count) as _;
        let mut activate_jobs_stream = self.client.activate_jobs(activate_jobs);
        loop {
            match activate_jobs_stream.next().await {
                Some(Ok(ActivatedJobs { activated_jobs })) => {
                    let it = activated_jobs.into_iter().map(|activated_job| {
                        async {
                            self.job_count.fetch_add(1, Ordering::SeqCst);
                            let job_key: i64 = activated_job.key;
                            let retries = activated_job.retries;
                            match AssertUnwindSafe((self.job_handler)(activated_job))
                                .catch_unwind()
                                .await
                            {
                                Ok(JobResult::NoAction) => {}
                                Ok(JobResult::Complete { variables }) => {
                                    let complete_job = CompleteJob { job_key, variables };
                                    self.client.complete_job(complete_job).await.unwrap();
                                }
                                Ok(JobResult::Fail { .. }) => {
                                    self.client.fail_job(job_key, retries - 1).await.unwrap();
                                }
                                Err(_) => match self.panic_option {
                                    PanicOption::DoNothingOnPanic => {}
                                    PanicOption::FailJobOnPanic => {
                                        self.client.fail_job(job_key, retries - 1).await.unwrap();
                                    }
                                },
                            };
                            self.job_count.fetch_sub(1, Ordering::SeqCst);
                        }
                    });
                    futures::future::join_all(it).await;
                }
                Some(Err(e)) => {
                    println!("there was a problem activating jobs, {:?}", e);
                    break;
                }
                None => {
                    break;
                }
            }
        }
    }
}

/// A worker will activate and process jobs with a job handler function.
/// The job worker has a maximum number of concurrent jobs that is controlled by an atomic,
/// consistent counter. When a job handler returns, the job worker will respond to the gateway
/// with the correct response (complete or fail). The job worker can be configured with a panic
/// option, so that it knows if to fail the job or do nothing if the handler panics.
///
/// The only method will poll for jobs, and if there are any, will start processing jobs. It will
/// only activate up to the maximum number of concurrent jobs allowed. When jobs complete, the counter
/// is updated.
#[derive(Clone)]
pub struct JobWorker {
    job_internal: Arc<JobInternal>,
}

impl JobWorker {
    pub fn new<F>(
        worker: String,
        job_type: String,
        timeout: i64,
        max_amount: u16,
        panic_option: PanicOption,
        client: Client,
        job_handler: F,
    ) -> Self
    where
        F: Fn(ActivatedJob) -> Pin<Box<dyn Future<Output = JobResult> + Send>>
            + Send
            + Sync
            + 'static,
    {
        let job_internal = Arc::new(JobInternal {
            job_handler: Box::new(job_handler),
            job_count: AtomicUsize::new(0),
            max_concurrent_jobs: max_amount as _,
            client,
            worker_name: worker,
            job_type,
            timeout,
            panic_option,
        });

        JobWorker { job_internal }
    }

    pub async fn activate_and_process_jobs(self) {
        self.job_internal.activate_and_process_jobs().await;
    }
}

//
//#[cfg(test)]
//mod test {
//    use crate::gateway_mock::MockGatewayClient;
//    use crate::worker::{JobResult, JobWorker, PanicOption};
//    use crate::ActivatedJob;
//    use futures::{Future, Stream};
//    use futures_cpupool::CpuPool;
//    use std::sync::Arc;
//
//    #[test]
//    fn completing_jobs() {
//        let mock_gateway_client = Arc::new(MockGatewayClient { jobs: vec![1, 2] });
//
//        let handler = |_aj: ActivatedJob| {
//            futures::future::ok::<JobResult, String>(JobResult::Complete { variables: None })
//        };
//
//        let pool = CpuPool::new(1);
//
//        let mut job_worker = JobWorker::new(
//            "rusty-worker".to_string(),
//            "a".to_string(),
//            10000,
//            32,
//            PanicOption::FailJobOnPanic,
//            mock_gateway_client.clone(),
//            handler,
//            pool,
//        );
//
//        let results: Vec<(JobResult, i64)> = job_worker
//            .activate_and_process_jobs()
//            .collect()
//            .wait()
//            .unwrap()
//            .into_iter()
//            .map(|(results, a)| (results, a.key))
//            .collect();
//
//        assert_eq!(
//            results,
//            vec![
//                (JobResult::Complete { variables: None }, 1i64),
//                (JobResult::Complete { variables: None }, 2i64)
//            ]
//        );
//    }
//
//    #[test]
//    fn failing_jobs() {
//        let mock_gateway_client = Arc::new(MockGatewayClient { jobs: vec![1, 2] });
//
//        let handler = |_aj: ActivatedJob| {
//            futures::future::ok::<JobResult, String>(JobResult::Fail {
//                error_message: None,
//            })
//        };
//
//        let pool = CpuPool::new(1);
//
//        let mut job_worker = JobWorker::new(
//            "rusty-worker".to_string(),
//            "a".to_string(),
//            10000,
//            32,
//            PanicOption::FailJobOnPanic,
//            mock_gateway_client.clone(),
//            handler,
//            pool,
//        );
//
//        let results: Vec<(JobResult, i64)> = job_worker
//            .activate_and_process_jobs()
//            .collect()
//            .wait()
//            .unwrap()
//            .into_iter()
//            .map(|(results, a)| (results, a.key))
//            .collect();
//
//        assert_eq!(
//            results,
//            vec![
//                (
//                    JobResult::Fail {
//                        error_message: None
//                    },
//                    1i64
//                ),
//                (
//                    JobResult::Fail {
//                        error_message: None
//                    },
//                    2i64
//                )
//            ]
//        );
//    }
//
//    #[test]
//    fn some_jobs_panic() {
//        let mock_gateway_client = Arc::new(MockGatewayClient { jobs: vec![1, 2] });
//
//        let handler = |aj: ActivatedJob| {
//            if aj.key == 1 {
//                panic!("gahh!");
//            }
//            futures::future::ok::<JobResult, String>(JobResult::Complete { variables: None })
//        };
//
//        let pool = CpuPool::new(1);
//
//        let mut job_worker = JobWorker::new(
//            "rusty-worker".to_string(),
//            "a".to_string(),
//            10000,
//            32,
//            PanicOption::DoNothingOnPanic,
//            mock_gateway_client.clone(),
//            handler,
//            pool,
//        );
//
//        let results: Vec<(JobResult, i64)> = job_worker
//            .activate_and_process_jobs()
//            .collect()
//            .wait()
//            .unwrap()
//            .into_iter()
//            .map(|(results, a)| (results, a.key))
//            .collect();
//
//        assert_eq!(
//            results,
//            vec![
//                (JobResult::NoAction, 1i64),
//                (JobResult::Complete { variables: None }, 2i64)
//            ]
//        );
//    }
//
//    #[test]
//    fn some_jobs_panic_with_fail_option() {
//        let mock_gateway_client = Arc::new(MockGatewayClient { jobs: vec![1, 2] });
//
//        let handler = |aj: ActivatedJob| {
//            if aj.key == 2 {
//                panic!("gahh!");
//            }
//            futures::future::ok::<JobResult, String>(JobResult::Complete { variables: None })
//        };
//
//        let pool = CpuPool::new(1);
//
//        let mut job_worker = JobWorker::new(
//            "rusty-worker".to_string(),
//            "a".to_string(),
//            10000,
//            32,
//            PanicOption::FailJobOnPanic,
//            mock_gateway_client.clone(),
//            handler,
//            pool,
//        );
//
//        let results: Vec<JobResult> = job_worker
//            .activate_and_process_jobs()
//            .collect()
//            .wait()
//            .unwrap()
//            .into_iter()
//            .map(|(results, _)| results)
//            .collect();
//
//        assert_eq!(
//            results,
//            vec![
//                JobResult::Complete { variables: None },
//                JobResult::Fail {
//                    error_message: Some("Job Handler Panicked.".to_string())
//                },
//            ]
//        );
//    }
//}
