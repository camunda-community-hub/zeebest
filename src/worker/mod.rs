use std::time::Duration;

use futures::{Future, IntoFuture, Stream};

use crate::gateway;
use crate::gateway_grpc;
use crate::worker::activate_jobs::ActivateJobs;
use crate::worker::job::Job;
use crate::worker::poll::Poll;
use crate::client::Client;

pub mod activate_jobs;
pub mod complete_job;
pub mod job;
pub mod job_worker;
pub mod poll;

pub struct JobsConfig {
    pub worker: String,
    pub job_type: String,
    pub timeout: i64,
    pub amount: i32,
}

pub fn do_work(poll_period: Duration, worker_config: JobsConfig) {
    let worker_client = Client::new().unwrap();

    Poll::new(poll_period).map(|_tick| {

        // spawn stream that will do all of the necessary things
        // force the stream into a future so it may be spawned
        let activated_jobs = worker_client.activate_jobs(&worker_config);
        let _spawn_jobs = activated_jobs.map_err(|_| ()).and_then(spawn_jobs);
    });
}

pub fn spawn_jobs(activated_job: gateway::ActivatedJob) -> impl IntoFuture<Item = (), Error = ()> {
    let job_key = activated_job.key;
    let payload = activated_job.payload;



    let job = Job::new(job_key, payload, |_, _| futures::future::ok(()));
    tokio::spawn(job)
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
