use crate::activate_jobs::ActivateJobsConfig;
use crate::client::Client;
use crate::complete_job::CompletedJobData;
use crate::gateway::ActivatedJob;

use futures::{Async, Future, Stream};
use std::sync::Arc;
use std::time::Duration;
use tokio::timer::Interval;

pub mod job_worker;

/// A basic usage of the client API for activating jobs and completing them.
pub fn do_work(poll_period: Duration, activate_jobs_config: ActivateJobsConfig) {
    let client = Client::new().unwrap();
    Interval::new_interval(poll_period)
        .zip(futures::stream::repeat(client))
        .map(|(_tick, client)| {
            // spawn stream that will do all of the necessary things
            // force the stream into a future so it may be spawned
            let work = client
                .activate_jobs(&activate_jobs_config)
                .and_then(|_activated_job| {
                    // do work
                    Ok(())
                })
                .zip(futures::stream::repeat(client))
                .and_then(|(_data, client)| {
                    let completed_job_data = CompletedJobData {
                        job_key: 1,
                        payload: None,
                    };
                    client.complete_job(completed_job_data)
                })
                .for_each(|_| Ok(()))
                .map_err(|_| ());
            tokio::spawn(work);
        });
}

/// The same application but using the nicer worker API
pub fn do_work_better(_poll_period: Duration, activate_jobs_config: ActivateJobsConfig) {
    let client = Client::new().unwrap();
    let client = Arc::new(client);

    let handler = |_key, _payload| None;

    let worker_config = WorkerConfig {
        activate_jobs_config,
        cancel_workflow_on_panic: false,
    };

    let worker = Worker::new(client, worker_config, handler);

    tokio::run(
        worker
            .map(|x| println!("completed job: {:?}", x))
            .map_err(|e| println!("error! {:?}", e))
            .collect()
            .then(|_| Ok(())),
    );
}

pub struct WorkerConfig {
    pub activate_jobs_config: ActivateJobsConfig,
    pub cancel_workflow_on_panic: bool,
}

pub struct Worker {
    s: Box<dyn Stream<Item = CompletedJobData, Error = grpc::Error> + Send>,
}

impl Worker {
    fn new<F>(client: Arc<Client>, worker_config: WorkerConfig, f: F) -> Self
    where
        F: Fn(i64, String) -> Option<String> + Send + 'static,
    {
        let on_activated_job = move |activated_job: ActivatedJob| {
            // do work
            let job_key = activated_job.key;
            let payload = activated_job.payload;
            Ok((job_key, f(job_key, payload)))
        };

        let work = client
            .activate_jobs(&worker_config.activate_jobs_config)
            .and_then(on_activated_job)
            .zip(futures::stream::repeat(client))
            .and_then(|(data, client)| {
                let completed_job_data = CompletedJobData {
                    job_key: data.0,
                    payload: data.1,
                };
                client.complete_job(completed_job_data)
            });
        Worker { s: Box::new(work) }
    }
}

impl Stream for Worker {
    type Item = CompletedJobData;
    type Error = grpc::Error;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        self.s.poll()
    }
}
