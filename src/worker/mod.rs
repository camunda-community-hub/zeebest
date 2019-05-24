pub mod activate_jobs;
pub mod complete_job;
pub mod job;
pub mod poll;

use crate::client::Client;
use crate::worker::activate_jobs::ActivateJobs;
use crate::worker::poll::{Poll, Tick};
use futures::{Future, Stream};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::timer::Interval;

struct WorkData {}

pub struct JobType {
    job_type: String,
}

pub struct WorkerType {
    worker_type: String,
}

pub fn do_work(job_type: Arc<JobType>, worker_type: Arc<WorkerType>) {
    Poll::new(Duration::from_millis(1000)).map(|tick| {
        // spawn stream that will do all of the necessary things
        // force the stream into a future so it may be spawned
        let stream = ActivateJobs::new(job_type.clone(), worker_type.clone()).for_each(|_| Ok(()));
        tokio::spawn(stream);
    });
}

pub struct Worker {
    max_active_jobs: usize,
    poll_interval: Interval,
    client: Arc<RwLock<Client>>,
}

impl Worker {
    pub fn new(max_active_jobs: u8, poll_period: Duration) -> Self {
        let poll_interval = Interval::new(Instant::now(), poll_period);
        let active_jobs_count = Arc::new(RwLock::new(0));
        let client = Arc::new(RwLock::new(Client::new().unwrap()));
        Worker {
            max_active_jobs: max_active_jobs as usize,
            poll_interval,
            client,
        }
    }
}
