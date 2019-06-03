use std::sync::{Arc, RwLock};
// requires future and stream
use futures::{Future, Stream};
use std::time::Duration;
use zeebest::{ActivateJobsConfig, Client, JobFn, JobResponse, PanicOption, WorkerConfig};

fn main() {
    // put the client in an Arc because it will be used on different threads
    let client = Client::new("127.0.0.1", 26500).unwrap();

    let job_fn = JobFn::with_job_type_and_handler("payment-service", move |_payload| {
        JobResponse::Complete { payload: None }
    })
    .panic_option(PanicOption::DoNothingOnPanic);

    let worker = client
        .worker("rusty-worker")
        .default_timeout(1000)
        .default_amount(32)
        .default_panic_option(PanicOption::FailJobOnPanic)
        .job(
            JobFn::with_job_type_and_handler("payment-service", move |_payload| {
                JobResponse::Complete { payload: None }
            })
            .panic_option(PanicOption::DoNothingOnPanic)
            .amount(2),
        );

    let job_stream = worker.into_job_stream();

    // poll on an interval, just do the same thing over and over
//    let future = client
//        .activate_and_process_jobs_interval(Duration::from_millis(1000), worker_config, job_fn)
//        .map_err(|e| println!("error doing job. {:?}", e))
//        .map(|c| println!("completed job. {:?}", c))
//        .or_else(|_| Ok(()))
//        .collect()
//        .map(|_| ());

//    tokio::run(future);
}
