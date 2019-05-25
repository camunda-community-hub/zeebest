use std::sync::Arc;
use std::time::Duration;
use zeebe_client::activate_jobs::ActivateJobsConfig;
use zeebe_client::client::Client;
use zeebe_client::worker::WorkerConfig;

// requires future and stream
use futures::{Future, Stream};
use tokio::timer::Interval;

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

    // this is your work function
    let handler = |_key, _payload| {
        // does some work...
        let _x = 10 * 10;
        // returns no payload
        None
    };

    // poll on an interval, just do the same thing over and over
    let future = Interval::new_interval(Duration::from_millis(1000))
        .map_err(|e| panic!("timer error {:?}", e))
        // make the worker available to future by zipping with it
        .zip(futures::stream::repeat((client, worker_config, handler)))
        // on each tick, create the worker task
        .and_then(|(_tick, (client, config, handler))| {
            // spawn stream that will do all of the necessary things
            // force the stream into a future so it may be spawned
            println!("spawning work!");
            let work = client
                .worker(config, handler)
                .for_each(|_| {
                    println!("doing work");
                    Ok(())
                })
                .map_err(|_| {
                    println!("work error!");
                });
            work
        })
        .collect()
        .map_err(|_| ())
        .map(|_| ());

    tokio::run(future);
}
