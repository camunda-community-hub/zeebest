// requires future and stream
use futures::{Future, Stream};
use std::time::Duration;
use zeebest::{ActivateJobsConfig, Client, JobError, WorkerConfig};

fn main() {
    // put the client in an Arc because it will be used on different threads
    let client = Client::new("127.0.0.1", 26500).unwrap();

    // define some information about the worker and the what it will do
    let worker_config = WorkerConfig {
        activate_jobs_config: ActivateJobsConfig {
            worker: "rusty-worker".to_string(),
            job_type: "payment-service".to_string(),
            timeout: 1000,
            amount: 32,
        },
        cancel_workflow_on_panic: false,
    };

    // this is your work function - this one always errors!
    let handler = move |_payload| {
        Err(JobError::Retry {
            msg: "There was a problem! Please retry. ".to_string(),
        })
    };

    // poll on an interval, just do the same thing over and over
    let future = client
        .activate_and_process_jobs_interval(Duration::from_millis(1000), worker_config, handler)
        .map_err(|e| println!("error doing job. {:?}", e))
        .map(|c| println!("completed job. {:?}", c))
        .or_else(|_| Ok(()))
        .collect()
        .map(|_| ());

    tokio::run(future);
}
