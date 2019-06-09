use crate::gateway;
use crate::gateway_grpc;
use crate::gateway_grpc::Gateway;
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
#[derive(Clone)]
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
    gateway_client: Arc<gateway_grpc::GatewayClient>,
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
        gateway_client: Arc<gateway_grpc::GatewayClient>,
        handler: H,
    ) -> Self {
        assert!(max_amount > 0, "max amount must be greater than zero");

        // the current number of running jobs, initilized to zero, and wrapped in an atomic because
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
        assert!(current_amount < self.max_amount, "current number of jobs exceeds allowed number of running jobs");

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
                let difference = (job_count as u16) - next_amount;
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
                    ((handler)(a)) // call the handler with the `ActivatedJob` data
                        .map_err(|e| Error::JobError(e))
                        // catch any panic, and handle the panic according to the panic option
                        .catch_unwind()
                        .then(move |r: Result<Result<JobResult, Error>, _>| match r {
                            // all non panics are simply unwrapped to the underlying result
                            Ok(job_result) => job_result,
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

#[cfg(test)]
mod test {
    use crate::gateway;
    use crate::worker::{JobResult, JobWorker, PanicOption};
    use futures::{Future, Stream};
    use std::time::Duration;
    use tokio::timer::Interval;

    /// THIS DEMONSTRATES USAGE, NOT ACTUALLY A TEST
    #[test]
    fn do_stuff() {
        use crate::Client;
        let client = Client::new("127.0.0.1", 26500).unwrap();

        // handler for job type "foo"
        let handler_foo = |_aj: gateway::ActivatedJob| {
            futures::future::ok::<JobResult, String>(JobResult::Complete { payload: None })
        };

        // handler for job type "bar"
        let handler_bar = |_aj: gateway::ActivatedJob| {
            futures::future::ok::<JobResult, String>(JobResult::Complete { payload: None })
        };

        let mut job_worker_a = JobWorker::new(
            "rusty-worker".to_string(),
            "a".to_string(),
            10000,
            32,
            PanicOption::FailJobOnPanic,
            client.gateway_client.clone(),
            handler_foo,
        );

        let mut job_worker_b = JobWorker::new(
            "rusty-worker".to_string(),
            "b".to_string(),
            10000,
            1,
            PanicOption::FailJobOnPanic,
            client.gateway_client.clone(),
            handler_bar,
        );

        // just get jobs right now, do nothing with stream
        let job_a_stream = job_worker_a.activate_and_process_jobs();

        // get jobs on an interval, throw on tokio
        // may be able to make this more convenient, but you get the point
        let do_work_on_interval = Interval::new_interval(Duration::from_millis(1000))
            .map_err(|_| ())
            .and_then(move |_| {
                job_worker_b
                    .activate_and_process_jobs()
                    .collect()
                    .map_err(|_| ())
            })
            .collect()
            .map(|_| ());

        tokio::run(do_work_on_interval);
    }

}
