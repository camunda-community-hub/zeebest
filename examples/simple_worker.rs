use std::sync::{Arc, RwLock};
use zeebe_client::activate_jobs::ActivateJobsConfig;
use zeebe_client::client::Client;

// requires future and stream
use futures::{Future, Stream};
use zeebe_client::activate_and_process_jobs::WorkerConfig;

fn main() {
    // put the client in an Arc because it will be used on different threads
    let client = Arc::new(Client::new().unwrap());

    // define some information about the worker and the what it will do
    let worker_config = WorkerConfig {
        activate_jobs_config: ActivateJobsConfig {
            worker: "my-worker".to_string(),
            job_type: "payment-processor".to_string(),
            timeout: 2000,
            amount: 5,
        },
        cancel_workflow_on_panic: false,
    };

    let data = Arc::new(RwLock::new(10));

    let handler = move |_payload| {
        // does some work...
        let _x = 10 * 10;

        *data.write().unwrap() += 5;
        // returns no payload
        Ok(None)
    };
    // this is your work function

    // poll on an interval, just do the same thing over and over
    let future = client
        .worker(worker_config, handler)
        .collect()
        .map_err(|_| ())
        .map(|_| ());

    tokio::run(future);
}
