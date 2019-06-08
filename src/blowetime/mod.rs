//use futures::Future;
//
//trait JobFactory {
//    type Job;
//
//    type Future: Future<Item = Self::Job, Error = ()>;
//
//    fn new_job() -> Self::Future;
//}
//
//trait RegisterJobFactory {
//    fn register(self, registrar: &mut JobFactoryRegistrar);
//}
//
//struct JobFactoryRegistrar {
//    job_factories: Vec<Box<RegisterJobFactory>>,
//}
//
//impl JobFactoryRegistrar {
//    fn register_factory(&mut self) {
//
//    }
//}
//

use std::sync::{Arc};
use futures::{Stream, Future};
use crate::gateway;
use crate::gateway_grpc;
use crate::gateway_grpc::Gateway;
use std::sync::atomic::{AtomicU16, Ordering};

struct JobConfig {
    pub amount: u16,
}

struct WorkerConfig {
    pub job_type: String,
    pub max_amount: u16,
}

struct Job;

enum JobResult {
    Complete{payload: Option<String>},
    Fail,
}

struct JobHandler {
}

struct JobWorker<H, F>
where
    H: Fn(gateway::ActivatedJob) -> F,
    F: Future<Item = JobResult, Error = ()>,
{
    worker: String,
    job_type: String,
    timeout: i64,
    current_amount: Arc<AtomicU16>,
    max_amount: u16,
    gateway_client: Arc<gateway_grpc::GatewayClient>,
    handler: Arc<H>
}

impl<H, F> JobWorker<H, F>
    where
        H: Fn(gateway::ActivatedJob) -> F,
        F: Future<Item = JobResult, Error = ()>,
{
    pub fn new(worker: String, job_type: String, timeout: i64,
               max_amount: u16,
               gateway_client: Arc<gateway_grpc::GatewayClient>,
               handler: H,) -> Self {

        assert!(max_amount > 0, "max amount must be greater than zero");

        let current_amount = Arc::new(AtomicU16::new(0));

        let handler = Arc::new(handler);

        JobWorker {
            worker,
            job_type,
            timeout,
            current_amount,
            max_amount,
            gateway_client,
            handler,
        }
    }

    pub fn activate_and_process_jobs(&mut self) {
        let current_number_of_jobs = self.current_amount.clone();
        let current_number_of_jobs_2 = self.current_amount.clone();

        let current_amount = current_number_of_jobs.load(Ordering::SeqCst);
        if current_amount > self.max_amount {
            unreachable!("current number of jobs exceeds allowed number of running jobs");
        }

        // also inspect the job count, so as to only request a number of jobs equal to available slots
        let next_amount = self.max_amount - current_amount;
        // increment current amount by next amount. This effectively locks the available job slots.
        // This is needed in case the requests are very slow, and/or do not return in order they were sent.
        current_number_of_jobs.fetch_add(next_amount, Ordering::SeqCst);

        let handler = self.handler.clone();

        let mut activate_jobs_request = gateway::ActivateJobsRequest::default();
        activate_jobs_request.set_amount(next_amount as i32); // TODO: make this configurable
        activate_jobs_request.set_timeout(self.timeout);
        activate_jobs_request.set_worker(self.worker.clone());
        activate_jobs_request.set_field_type(self.job_type.clone());
        let options = Default::default();
        let grpc_response: grpc::StreamingResponse<_> = self.gateway_client.activate_jobs(options, activate_jobs_request);

        let grpc_stream = grpc_response.drop_metadata();

        let jobs_stream = grpc_stream
            .and_then(move |response| {
                Ok((response.jobs.len(),response.jobs.into_iter()))
            })
            .and_then(move |(job_count, jobs)| {
                assert!(job_count < (std::u16::MAX as usize), "maximum job count exceeded maximum number of allowed concurrent jobs");
                // the counter has already been incremented, but there may be fewer jobs returned.
                // Decrease the counter if the difference is greater than zero, this opens
                // available jobs slots.
                let difference = (job_count as u16) - next_amount;
                if difference > 0 {
                    current_number_of_jobs.fetch_sub(difference, Ordering::SeqCst);
                }
                Ok(jobs)
            })
            // call the handler for each activated job, return collection of futures
            .map(|jobs| {
                let handler = handler.clone();
                jobs.map(move |a| {
                    (handler)(a)
                })
            })
            // create an stream of futures that is unordered - yields as futures complete
            .map(|fs| futures::stream::futures_unordered(fs))
            // decrement the job count as jobs complete
            .inspect(move|_| {
                current_number_of_jobs_2.fetch_sub(1, Ordering::SeqCst);
            })
            ;
    }
}

fn do_stuff() {
    use crate::Client;
    let client = Client::new("127.0.0.1", 26500).unwrap();

    // handler for job type "foo"
    let handler_foo = |_aj: gateway::ActivatedJob| {
        futures::future::ok::<JobResult, ()>(JobResult::Complete {payload: None})
    };

    // foo jobs will use the default maximum number of concurrent jobs

    // handler for job type "bar"
    let handler_bar = |_aj: gateway::ActivatedJob| {
        futures::future::ok::<JobResult, ()>(JobResult::Complete {payload: None})
    };

    let mut job_worker_a = JobWorker::new("rusty-worker".to_string(), "a".to_string(), 10000, 32, client.gateway_client.clone(), handler_foo);

    let job_worker_b = JobWorker::new("rusty-worker".to_string(),"b".to_string(), 10000, 1, client.gateway_client.clone(), handler_bar);

    let job_a_stream = job_worker_a.activate_and_process_jobs();
}
