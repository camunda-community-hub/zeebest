//use crate::gateway_grpc;
//use crate::Error;
//use crate::{gateway, ActivatedJob};
//use futures::{Future, IntoFuture, Poll, Stream};
//use futures_cpupool::CpuPool;
//use std::panic::AssertUnwindSafe;
//use std::sync::atomic::{AtomicU16, Ordering};
//use std::sync::Arc;
//
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WorkerConfig {
    /// the name of the worker activating the jobs, mostly used for logging purposes
    pub worker_name: String,
    /// the job type, as defined in the BPMN process (e.g. <zeebe:taskDefinition type="payment-service" />)
    pub job_type: String,
    /// a job returned after this call will not be activated by another call until the timeout has been reached
    pub timeout: i64,
    /// the maximum concurrent number of jobs
    pub max_concurrent_jobs: i32,
    /// panic handling option
    pub panic_option: PanicOption,
}

impl WorkerConfig {
    pub fn new(worker: String, job_type: String, timeout: i64, max_jobs_to_activate: i32, panic_option: PanicOption) -> Self {
        Self {
            worker_name: worker,
            job_type,
            timeout,
            max_concurrent_jobs: max_jobs_to_activate,
            panic_option,
        }
    }
}

//
///// A tri-either type for nicely wrapping the final result which branches based on the response to
///// the grpc gateway.
//enum JobResultFuture<C, F, N> {
//    Complete(C),
//    Fail(F),
//    NoAction(N),
//}
//
//impl<C, F, N> Future for JobResultFuture<C, F, N>
//where
//    C: Future,
//    F: Future<Item = C::Item, Error = C::Error>,
//    N: Future<Item = C::Item, Error = C::Error>,
//{
//    type Item = C::Item;
//    type Error = C::Error;
//
//    fn poll(&mut self) -> Poll<C::Item, C::Error> {
//        match *self {
//            JobResultFuture::Complete(ref mut c) => c.poll(),
//            JobResultFuture::Fail(ref mut f) => f.poll(),
//            JobResultFuture::NoAction(ref mut n) => n.poll(),
//        }
//    }
//}
//
///// A worker will activate and process jobs with a job handler function.
///// The job worker has a maximum number of concurrent jobs that is controlled by an atomic,
///// consistent counter. When a job handler returns, the job worker will respond to the gateway
///// with the correct response (complete or fail). The job worker can be configured with a panic
///// option, so that it knows if to fail the job or do nothing if the handler panics.
/////
///// The only method will poll for jobs, and if there are any, will start processing jobs. It will
///// only activate up to the maximum number of concurrent jobs allowed. When jobs complete, the counter
///// is updated.
//pub struct JobWorker<H, F>
//where
//    H: Fn(ActivatedJob) -> F + Send + Sync + 'static,
//    F: IntoFuture<Item = JobResult, Error = String> + 'static,
//    <F as IntoFuture>::Future: Send + 'static,
//    <F as IntoFuture>::Item: Send + 'static,
//    <F as IntoFuture>::Error: Send + 'static,
//{
//    worker: String,
//    job_type: String,
//    timeout: i64,
//    current_amount: Arc<AtomicU16>,
//    max_amount: u16,
//    panic_option: PanicOption,
//    gateway_client: Arc<gateway_grpc::Gateway + Send + Sync>,
//    handler: Arc<H>,
//    thread_pool: CpuPool,
//}
//
//impl<H, F> JobWorker<H, F>
//where
//    H: Fn(ActivatedJob) -> F + Send + Sync + 'static,
//    F: IntoFuture<Item = JobResult, Error = String> + 'static,
//    <F as IntoFuture>::Future: Send + 'static,
//    <F as IntoFuture>::Item: Send + 'static,
//    <F as IntoFuture>::Error: Send + 'static,
//{
//    pub(crate) fn new(
//        worker: String,
//        job_type: String,
//        timeout: i64,
//        max_amount: u16,
//        panic_option: PanicOption,
//        gateway_client: Arc<gateway_grpc::Gateway + Send + Sync>,
//        handler: H,
//        thread_pool: CpuPool,
//    ) -> Self {
//        assert!(max_amount > 0, "max amount must be greater than zero");
//
//        // the current number of running jobs, initialized to zero, and wrapped in an atomic because
//        // this value will be modified between threads
//        let current_amount = Arc::new(AtomicU16::new(0));
//
//        // put the handler in an arc, so it may be easily accessed in many threads
//        let handler = Arc::new(handler);
//
//        JobWorker {
//            worker,
//            job_type,
//            timeout,
//            current_amount,
//            max_amount,
//            panic_option,
//            gateway_client,
//            handler,
//            thread_pool,
//        }
//    }
//
//    /// Activate jobs of the `job_type` and no more than the `max_amount`. The activate jobs
//    /// will be processed by the `handler`.
//    pub fn activate_and_process_jobs(
//        &mut self,
//    ) -> impl Stream<Item = (JobResult, ActivatedJob), Error = Error> {
//        // clone the job counter, as it will be moved into a few different closures
//        let current_number_of_jobs = self.current_amount.clone();
//        let current_number_of_jobs_2 = self.current_amount.clone();
//
//        // read the current job count value
//        let current_amount = current_number_of_jobs.load(Ordering::SeqCst);
//
//        // assert on an unreachable edge case
//        assert!(
//            current_amount <= self.max_amount,
//            "current number of jobs exceeds allowed number of running jobs"
//        );
//
//        // also inspect the job count, so as to only request a number of jobs equal to available slots
//        let next_max_jobs_to_activate = self.max_amount - current_amount;
//
//        // construct a request - this is all grpc stuff
//        let mut activate_jobs_request = gateway::ActivateJobsRequest::default();
//        activate_jobs_request.set_maxJobsToActivate(next_max_jobs_to_activate as i32); // TODO: make this configurable
//        activate_jobs_request.set_timeout(self.timeout);
//        activate_jobs_request.set_worker(self.worker.clone());
//        activate_jobs_request.set_field_type(self.job_type.clone());
//        let options = Default::default();
//
//        // create the grpc stream response
//        let grpc_response: grpc::StreamingResponse<_> = self
//            .gateway_client
//            .activate_jobs(options, activate_jobs_request);
//
//        // drop the unnecessary grpc metadata, and convert the error
//        let grpc_stream = grpc_response
//            .drop_metadata()
//            .map_err(|e| Error::ActivateJobError(e));
//
//        // make a copy of the panic option
//        let panic_option = self.panic_option;
//
//        // make a clone of the gateway client
//        let gateway_client = self.gateway_client.clone();
//
//        // start building the jobs stream
//        let jobs_stream = grpc_stream
//            .and_then(move |response| Ok((response.jobs.len(), response.jobs.into_iter())))
//            .and_then(move |(job_count, jobs)| {
//                assert!(
//                    job_count < (std::u16::MAX as usize),
//                    "maximum job count exceeded maximum number of allowed concurrent jobs"
//                );
//                current_number_of_jobs.fetch_add(job_count as u16, Ordering::SeqCst);
//                Ok(jobs)
//            })
//            // insert a reference to the job handler fn into the stream
//            .zip(futures::stream::repeat((
//                self.handler.clone(),
//                self.thread_pool.clone(),
//            )))
//            // call the handler for each activated job, return collection of futures
//            .map(move |(jobs, (handler, thread_pool))| {
//                let panic_option = panic_option;
//                jobs.map(move |a| {
//                    let activated_job: ActivatedJob = a.clone().into();
//                    let handler = handler.clone();
//
//                    // call the handler with the `ActivatedJob` data and put it on the thread pool
//                    AssertUnwindSafe(
//                        thread_pool.spawn(futures::future::lazy(move || (handler)(a.into()))),
//                    )
//                    // catch any panic, and handle the panic according to the panic option
//                    .catch_unwind()
//                    .then(move |r: Result<Result<JobResult, _>, _>| match r {
//                        // all non panics are simply unwrapped to the underlying result
//                        Ok(job_result) => job_result.map_err(|e| Error::JobError(e)),
//                        // panic option is matched in case of a panic
//                        Err(_) => match panic_option {
//                            // TODO: capture the panic stacktrace and return it with the fail job request
//                            PanicOption::FailJobOnPanic => Ok(JobResult::Fail {
//                                error_message: Some("Job Handler Panicked.".to_string()),
//                            }),
//                            PanicOption::DoNothingOnPanic => Ok(JobResult::NoAction),
//                        },
//                    })
//                    .join(Ok(activated_job))
//                })
//            })
//            // create an stream of futures that is unordered - yields as futures complete
//            .map(futures::stream::futures_unordered)
//            // this resolves to a stream of streams, so it needs to be flattened
//            .flatten()
//            // decrement the job count as jobs complete
//            .inspect(move |_| {
//                current_number_of_jobs_2.fetch_sub(1, Ordering::SeqCst);
//            })
//            // finally, match on the item, and respond to the gateway
//            .and_then(move |(result, activated_job)| {
//                let result: JobResult = result;
//                let cloned_result_complete = result.clone();
//                let cloned_result_fail = result.clone();
//                let job_key = activated_job.key;
//                let retries = activated_job.retries;
//                match result {
//                    JobResult::NoAction => {
//                        JobResultFuture::NoAction(futures::future::ok((result, activated_job)))
//                    }
//                    JobResult::Fail { error_message } => {
//                        let options = Default::default();
//                        let mut fail_request = gateway::FailJobRequest::default();
//                        fail_request.set_jobKey(job_key);
//                        fail_request.set_retries(retries - 1);
//                        if let Some(error_message) = error_message {
//                            fail_request.set_errorMessage(error_message);
//                        }
//                        JobResultFuture::Fail(
//                            gateway_client
//                                .fail_job(options, fail_request)
//                                .drop_metadata()
//                                .map_err(|e| Error::FailJobError(e))
//                                .map(move |_| (cloned_result_complete, activated_job)),
//                        )
//                    }
//                    JobResult::Complete { variables } => {
//                        let options = Default::default();
//                        let mut complete_request = gateway::CompleteJobRequest::default();
//                        complete_request.set_jobKey(job_key);
//                        if let Some(variables) = variables {
//                            complete_request.set_variables(variables)
//                        }
//                        JobResultFuture::Complete(
//                            gateway_client
//                                .complete_job(options, complete_request)
//                                .drop_metadata()
//                                .map_err(|e| Error::CompleteJobError(e))
//                                .map(move |_| (cloned_result_fail, activated_job)),
//                        )
//                    }
//                }
//            });
//
//        jobs_stream
//    }
//}
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
