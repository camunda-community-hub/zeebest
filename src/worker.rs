use crate::gateway;
use crate::gateway_grpc;
use futures::{Future, Poll, Stream};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;

/// An option that describes what the job worker should do if if the job handler panics.
#[derive(Clone, Copy)]
pub enum PanicOption {
    FailJobOnPanic,
    DoNothingOnPanic,
}

/// A result that describes the output of a job.
#[derive(Clone, Debug, PartialEq)]
enum JobResult {
    Complete { payload: Option<String> },
    Fail,
    NoAction,
}

/// A tri-either type for nicely wrapping the final result which branches based on the response to
/// the grpc gateway.
enum JobResultFuture<C, F, N> {
    Complete(C),
    Fail(F),
    NoAction(N),
}

impl<C, F, N> Future for JobResultFuture<C, F, N>
where
    C: Future,
    F: Future<Item = C::Item, Error = C::Error>,
    N: Future<Item = C::Item, Error = C::Error>,
{
    type Item = C::Item;
    type Error = C::Error;

    fn poll(&mut self) -> Poll<C::Item, C::Error> {
        match *self {
            JobResultFuture::Complete(ref mut c) => c.poll(),
            JobResultFuture::Fail(ref mut f) => f.poll(),
            JobResultFuture::NoAction(ref mut n) => n.poll(),
        }
    }
}

#[derive(Debug, Fail)]
enum Error {
    #[fail(display = "Grpc Error")]
    GrpcError(grpc::Error),
    #[fail(display = "Job Error: {}", _0)]
    JobError(String),
}

/// The `JobWorker` describes what a job type and contains a job handler function to call when jobs
/// are ready to be worked on. The job worker has a maximum number of concurrent jobs that is
/// controlled by an atomic, consistent counter. When a job handler returns, the job worker will
/// respond to the gateway with the correct response (complete or fail). The job worker can be
/// configured with a panic option, so that it knows if to fail the job or do nothing if the handler
/// panics.
///
/// The only method will poll for jobs, and if there are any, will start processing jobs. It will
/// only activate up to the maximum number of concurrent jobs allowed. When jobs complete, the counter
/// is updated.
struct JobWorker<H, F>
where
    H: Fn(gateway::ActivatedJob) -> F,
    F: Future<Item = JobResult, Error = String> + std::panic::UnwindSafe,
{
    worker: String,
    job_type: String,
    timeout: i64,
    current_amount: Arc<AtomicU16>,
    max_amount: u16,
    panic_option: PanicOption,
    gateway_client: Arc<gateway_grpc::Gateway + Send + Sync>,
    handler: Arc<H>,
}

impl<H, F> JobWorker<H, F>
where
    H: Fn(gateway::ActivatedJob) -> F,
    F: Future<Item = JobResult, Error = String> + std::panic::UnwindSafe,
{
    pub fn new(
        worker: String,
        job_type: String,
        timeout: i64,
        max_amount: u16,
        panic_option: PanicOption,
        gateway_client: Arc<gateway_grpc::Gateway + Send + Sync>,
        handler: H,
    ) -> Self {
        assert!(max_amount > 0, "max amount must be greater than zero");

        // the current number of running jobs, initialized to zero, and wrapped in an atomic because
        // this value will be modified between threads
        let current_amount = Arc::new(AtomicU16::new(0));

        // put the handler in an arc, so it may be easily accessed in many threads
        let handler = Arc::new(handler);

        JobWorker {
            worker,
            job_type,
            timeout,
            current_amount,
            max_amount,
            panic_option,
            gateway_client,
            handler,
        }
    }

    pub fn activate_and_process_jobs(
        &mut self,
    ) -> impl Stream<Item = (JobResult, i64), Error = Error> {
        // clone the job counter, as it will be moved into a few different closures
        let current_number_of_jobs = self.current_amount.clone();
        let current_number_of_jobs_2 = self.current_amount.clone();

        // read the current job count value
        let current_amount = current_number_of_jobs.load(Ordering::SeqCst);

        // assert on an unreachable edge case
        assert!(
            current_amount < self.max_amount,
            "current number of jobs exceeds allowed number of running jobs"
        );

        // also inspect the job count, so as to only request a number of jobs equal to available slots
        let next_amount = self.max_amount - current_amount;
        // increment current amount by next amount. This effectively locks the available job slots.
        // This is needed in case the requests are very slow, and/or do not return in order they were sent.
        current_number_of_jobs.fetch_add(next_amount, Ordering::SeqCst);

        // construct a request - this is all grpc stuff
        let mut activate_jobs_request = gateway::ActivateJobsRequest::default();
        activate_jobs_request.set_amount(next_amount as i32); // TODO: make this configurable
        activate_jobs_request.set_timeout(self.timeout);
        activate_jobs_request.set_worker(self.worker.clone());
        activate_jobs_request.set_field_type(self.job_type.clone());
        let options = Default::default();

        // create the grpc stream response
        let grpc_response: grpc::StreamingResponse<_> = self
            .gateway_client
            .activate_jobs(options, activate_jobs_request);

        // drop the unnecessary grpc metadata, and convert the error
        let grpc_stream = grpc_response
            .drop_metadata()
            .map_err(|e| Error::GrpcError(e));

        // make a copy of the panic option
        let panic_option = self.panic_option;

        // make a clone of the gateway client
        let gateway_client = self.gateway_client.clone();

        // start building the jobs stream
        let jobs_stream = grpc_stream
            .and_then(move |response| Ok((response.jobs.len(), response.jobs.into_iter())))
            .and_then(move |(job_count, jobs)| {
                assert!(
                    job_count < (std::u16::MAX as usize),
                    "maximum job count exceeded maximum number of allowed concurrent jobs"
                );
                // the counter has already been incremented, but there may be fewer jobs returned.
                // Decrease the counter if the difference is greater than zero, this opens
                // available jobs slots.
                let difference = next_amount - (job_count as u16);
                if difference > 0 {
                    current_number_of_jobs.fetch_sub(difference, Ordering::SeqCst);
                }
                Ok(jobs)
            })
            // insert a reference to the job handler fn into the stream
            .zip(futures::stream::repeat(self.handler.clone()))
            // call the handler for each activated job, return collection of futures
            .map(move |(jobs, handler)| {
                let panic_option = panic_option;
                jobs.map(move |a| {
                    let job_key = a.get_key();
                    let retries = a.get_retries();
                    let f: F = (handler)(a); // call the handler with the `ActivatedJob` data
                        // catch any panic, and handle the panic according to the panic option
                        f.catch_unwind()
                        .then(move |r: Result<Result<JobResult, _>, _>| match r {
                            // all non panics are simply unwrapped to the underlying result
                            Ok(job_result) => job_result.map_err(|e| Error::JobError(e)),
                            // panic option is matched in case of a panic
                            Err(_panic_error) => match panic_option {
                                PanicOption::FailJobOnPanic => Ok(JobResult::Fail),
                                PanicOption::DoNothingOnPanic => Ok(JobResult::NoAction),
                            },
                        })
                        .join3(Ok(job_key), Ok(retries))
                })
            })
            // create an stream of futures that is unordered - yields as futures complete
            .map(futures::stream::futures_unordered)
            // this resolves to a stream of streams, so it needs to be flattened
            .flatten()
            // decrement the job count as jobs complete
            .inspect(move |_| {
                current_number_of_jobs_2.fetch_sub(1, Ordering::SeqCst);
            })
            // finally, match on the item, and respond to the gateway
            .and_then(move |(result, job_key, retries)| {
                let result: JobResult = result;
                let cloned_result_complete = result.clone();
                let cloned_result_fail = result.clone();
                let job_key: i64 = job_key;
                match result {
                    JobResult::NoAction => {
                        JobResultFuture::NoAction(futures::future::ok((result, job_key)))
                    }
                    JobResult::Fail => {
                        let options = Default::default();
                        let mut fail_request = gateway::FailJobRequest::default();
                        fail_request.set_jobKey(job_key);
                        fail_request.set_retries(retries - 1);
                        JobResultFuture::Fail(
                            gateway_client
                                .fail_job(options, fail_request)
                                .drop_metadata()
                                .map_err(|e| Error::GrpcError(e))
                                .map(move |_| (cloned_result_complete, job_key)),
                        )
                    }
                    JobResult::Complete { payload } => {
                        let options = Default::default();
                        let mut complete_request = gateway::CompleteJobRequest::default();
                        complete_request.set_jobKey(job_key);
                        if let Some(payload) = payload {
                            complete_request.set_payload(payload.clone())
                        }
                        JobResultFuture::Complete(
                            gateway_client
                                .complete_job(options, complete_request)
                                .drop_metadata()
                                .map_err(|e| Error::GrpcError(e))
                                .map(move |_| (cloned_result_fail, job_key)),
                        )
                    }
                }
            });

        Box::new(jobs_stream)
    }
}

struct MockGatewayClient {
    pub jobs: Vec<i64>,
}

impl gateway_grpc::Gateway for MockGatewayClient {
    fn topology(
        &self,
        o: grpc::RequestOptions,
        p: gateway::TopologyRequest,
    ) -> grpc::SingleResponse<gateway::TopologyResponse> {
        unimplemented!()
    }

    fn deploy_workflow(
        &self,
        o: grpc::RequestOptions,
        p: gateway::DeployWorkflowRequest,
    ) -> grpc::SingleResponse<gateway::DeployWorkflowResponse> {
        unimplemented!()
    }

    fn publish_message(
        &self,
        o: grpc::RequestOptions,
        p: gateway::PublishMessageRequest,
    ) -> grpc::SingleResponse<gateway::PublishMessageResponse> {
        unimplemented!()
    }

    fn create_job(
        &self,
        o: grpc::RequestOptions,
        p: gateway::CreateJobRequest,
    ) -> grpc::SingleResponse<gateway::CreateJobResponse> {
        unimplemented!()
    }

    fn update_job_retries(
        &self,
        o: grpc::RequestOptions,
        p: gateway::UpdateJobRetriesRequest,
    ) -> grpc::SingleResponse<gateway::UpdateJobRetriesResponse> {
        unimplemented!()
    }

    fn fail_job(
        &self,
        o: grpc::RequestOptions,
        p: gateway::FailJobRequest,
    ) -> grpc::SingleResponse<gateway::FailJobResponse> {
        let item: Box<
            dyn Future<Item = (gateway::FailJobResponse, grpc::Metadata), Error = grpc::Error>
                + Send
                + 'static,
        > = Box::new(futures::future::ok((
            gateway::FailJobResponse::default(),
            grpc::Metadata::default(),
        )));
        grpc::SingleResponse::new(futures::future::ok((grpc::Metadata::default(), item)))
    }

    fn complete_job(
        &self,
        o: grpc::RequestOptions,
        p: gateway::CompleteJobRequest,
    ) -> grpc::SingleResponse<gateway::CompleteJobResponse> {
        let item: Box<dyn Future<Item = _, Error = _> + Send + 'static> = Box::new(
            futures::future::ok((gateway::CompleteJobResponse::default(), Default::default())),
        );
        grpc::SingleResponse::new(futures::future::ok((Default::default(), item)))
    }

    fn create_workflow_instance(
        &self,
        o: grpc::RequestOptions,
        p: gateway::CreateWorkflowInstanceRequest,
    ) -> grpc::SingleResponse<gateway::CreateWorkflowInstanceResponse> {
        unimplemented!()
    }

    fn cancel_workflow_instance(
        &self,
        o: grpc::RequestOptions,
        p: gateway::CancelWorkflowInstanceRequest,
    ) -> grpc::SingleResponse<gateway::CancelWorkflowInstanceResponse> {
        unimplemented!()
    }

    fn update_workflow_instance_payload(
        &self,
        o: grpc::RequestOptions,
        p: gateway::UpdateWorkflowInstancePayloadRequest,
    ) -> grpc::SingleResponse<gateway::UpdateWorkflowInstancePayloadResponse> {
        unimplemented!()
    }

    fn activate_jobs(
        &self,
        o: grpc::RequestOptions,
        p: gateway::ActivateJobsRequest,
    ) -> grpc::StreamingResponse<gateway::ActivateJobsResponse> {
        let activated_jobs: Vec<_> = self
            .jobs
            .iter()
            .map(|job_key| {
                let mut activated_job = gateway::ActivatedJob::default();
                activated_job.set_retries(5);
                activated_job.set_key(*job_key);
                activated_job
            })
            .collect();
        let mut response = gateway::ActivateJobsResponse::default();
        response.set_jobs(activated_jobs.into());
        let activated_jobs = futures::stream::iter_ok(vec![response]);
        grpc::StreamingResponse::metadata_and_stream_and_trailing_metadata(
            Default::default(),
            activated_jobs,
            futures::future::ok(Default::default()),
        )
    }

    fn list_workflows(
        &self,
        o: grpc::RequestOptions,
        p: gateway::ListWorkflowsRequest,
    ) -> grpc::SingleResponse<gateway::ListWorkflowsResponse> {
        unimplemented!()
    }

    fn get_workflow(
        &self,
        o: grpc::RequestOptions,
        p: gateway::GetWorkflowRequest,
    ) -> grpc::SingleResponse<gateway::GetWorkflowResponse> {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use crate::gateway;
    use crate::worker::{JobResult, JobWorker, MockGatewayClient, PanicOption};
    use futures::{Future, Stream};
    use std::sync::Arc;

    #[test]
    fn completing_jobs() {
        let mock_gateway_client = Arc::new(MockGatewayClient { jobs: vec![1, 2] });

        let handler = |_aj: gateway::ActivatedJob| {
            futures::future::ok::<JobResult, String>(JobResult::Complete { payload: None })
        };

        let mut job_worker = JobWorker::new(
            "rusty-worker".to_string(),
            "a".to_string(),
            10000,
            32,
            PanicOption::FailJobOnPanic,
            mock_gateway_client.clone(),
            handler,
        );

        let results = job_worker
            .activate_and_process_jobs()
            .collect()
            .wait()
            .unwrap();

        assert_eq!(
            results,
            vec![
                (JobResult::Complete { payload: None }, 1i64),
                (JobResult::Complete { payload: None }, 2i64)
            ]
        );
    }

    #[test]
    fn failing_jobs() {
        let mock_gateway_client = Arc::new(MockGatewayClient { jobs: vec![1, 2] });

        let handler = |_aj: gateway::ActivatedJob| {
            futures::future::ok::<JobResult, String>(JobResult::Fail)
        };

        let mut job_worker = JobWorker::new(
            "rusty-worker".to_string(),
            "a".to_string(),
            10000,
            32,
            PanicOption::FailJobOnPanic,
            mock_gateway_client.clone(),
            handler,
        );

        let results = job_worker
            .activate_and_process_jobs()
            .collect()
            .wait()
            .unwrap();

        assert_eq!(
            results,
            vec![
                (JobResult::Fail, 1i64),
                (JobResult::Fail, 2i64)
            ]
        );
    }

    #[test]
    fn some_jobs_panic() {
        let mock_gateway_client = Arc::new(MockGatewayClient { jobs: vec![1, 2] });

        let handler = |aj: gateway::ActivatedJob| {
            if aj.key == 1 {
                panic!("gahh!");
            }
            futures::future::ok::<JobResult, String>(JobResult::Complete { payload: None})
        };

        let mut job_worker = JobWorker::new(
            "rusty-worker".to_string(),
            "a".to_string(),
            10000,
            32,
            PanicOption::DoNothingOnPanic,
            mock_gateway_client.clone(),
            handler,
        );

        let results = job_worker
            .activate_and_process_jobs()
            .collect()
            .wait()
            .unwrap();

        assert_eq!(
            results,
            vec![
                (JobResult::NoAction, 1i64),
                (JobResult::Complete { payload: None }, 2i64)
            ]
        );
    }
}
