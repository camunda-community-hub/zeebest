pub mod activate_jobs;
pub mod complete_job;
pub mod job;
pub mod poll;

use crate::client::Client;
use crate::gateway;
use crate::gateway_grpc;
use crate::worker::activate_jobs::ActivateJobs;
use crate::worker::job::Job;
use crate::worker::poll::{Poll, Tick};
use futures::{Future, IntoFuture, Stream};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::timer::Interval;

pub struct JobConfig {
    job_type: String,
    timeout: i64,
}

pub struct WorkerConfig {
    name: String,
    poll_period: Duration,
    client: gateway_grpc::GatewayClient,
}

pub struct JobWorker<F, H>
where
    F: Future<Item = Option<String>, Error = ()>,
    H: Fn(i64, String) -> F,
{
    name: String,
    job_type: String,
    max_running_jobs: Option<usize>,
    handler: H,
}

impl<F, H> JobWorker<F, H>
where
    F: Future<Item = Option<String>, Error = ()>,
    H: Fn(i64, String) -> F,
{
    pub fn new(name: String, job_type: String, handler: H) -> Self {
        JobWorker {
            name,
            job_type,
            max_running_jobs: None,
            handler,
        }
    }
}

pub fn do_work(job_config: JobConfig, worker_config: WorkerConfig) {
    Poll::new(worker_config.poll_period).map(|tick| {
        // spawn stream that will do all of the necessary things
        // force the stream into a future so it may be spawned
        let activated_jobs = ActivateJobs::new(&job_config, &worker_config);
        let spawn_jobs = activated_jobs.map_err(|_| ()).and_then(spawn_jobs);
    });
}

pub fn spawn_jobs(job: gateway::ActivatedJob) -> impl IntoFuture<Item = (), Error = ()> {
    let job_key = job.key;
    let payload = job.payload;
    let job = Job::new(job_key, payload, |_, _| futures::future::ok(()));
    tokio::spawn(job)
}

#[cfg(test)]
mod test {
    use crate::worker::JobWorker;

    #[test]
    fn make_worker() {
        let worker = JobWorker::new(
            "foo-worker".to_string(),
            "payment-process".to_string(),
            |key, payload| Ok(None),
        );
    }
}

//impl IntoFuture for JobWorker {
//    type Future = ();
//    type Item = ();
//    type Error = ();
//
//    fn into_future(self) -> Self::Future {
//        unimplemented!()
//    }
//}
