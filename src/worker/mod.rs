use std::time::Duration;

use futures::{Future, IntoFuture, Stream};

use crate::activate_jobs::ActivateJobsConfig;
use crate::client::Client;
use crate::gateway;
use crate::gateway_grpc;
use crate::worker::job::Job;
use crate::worker::poll::Poll;

pub mod job;
pub mod job_worker;
pub mod poll;

pub fn do_work(poll_period: Duration, activate_jobs_config: ActivateJobsConfig) {
    let worker_client = Client::new().unwrap();

    Poll::new(poll_period).map(|_tick| {
        // spawn stream that will do all of the necessary things
        // force the stream into a future so it may be spawned
        let activated_jobs = worker_client.activate_jobs(&activate_jobs_config);
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
