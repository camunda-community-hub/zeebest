use crate::activate_jobs::ActivateJobsConfig;
use crate::client::Client;
use crate::complete_job::CompletedJobData;
use crate::gateway::ActivatedJob;

use futures::{Async, Stream};
use std::sync::Arc;

#[derive(Clone)]
pub struct WorkerConfig {
    pub activate_jobs_config: ActivateJobsConfig,
    pub cancel_workflow_on_panic: bool,
}

pub struct Worker {
    s: Box<dyn Stream<Item = CompletedJobData, Error = grpc::Error> + Send>,
}

impl Worker {
    pub fn new<F>(client: Arc<Client>, worker_config: WorkerConfig, f: F) -> Self
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
