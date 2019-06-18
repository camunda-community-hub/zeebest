// requires future and stream
use futures::{Future, Stream};
use std::time::Duration;
use tokio::timer::Interval;
use zeebest::{Client, JobResult, PanicOption};

fn main() {
    let mut client = Client::new("127.0.0.1", 26500).unwrap();

    let handler = move |_payload| Ok(JobResult::Complete { variables: None });

    let mut worker = client.worker(
        "rusty-worker",
        "payment-service",
        Duration::from_secs(10),
        1,
        PanicOption::FailJobOnPanic,
        handler,
    );

    // poll on an interval, just do the same thing over and over
    let interval = Interval::new_interval(Duration::from_millis(2000))
        .map_err(|_| ())
        .and_then(move |_| {
            worker
                .activate_and_process_jobs()
                .and_then(|(result, activated_job)| {
                    println!("processed {} with result: {:?}", activated_job.key, result);
                    Ok(())
                })
                .collect()
                .map(|_| ()) // turn this stream into a future
                .map_err(|e| {
                    eprintln!("had error: {:?}", e);
                    ()
                })
        })
        .collect()
        .map(|_| ())
        .map_err(|_| ());

    tokio::run(interval);
}
