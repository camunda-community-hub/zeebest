use std::time::Duration;

use futures::{Future, IntoFuture, Stream};

use crate::activate_jobs::ActivateJobsConfig;
use crate::client::Client;
use crate::gateway;

use crate::complete_job::CompletedJobData;
use crate::worker::job::Job;
use crate::worker::poll::Poll;
use tokio::timer::Interval;

pub mod job;
pub mod job_worker;
pub mod poll;

pub fn do_work(poll_period: Duration, activate_jobs_config: ActivateJobsConfig) {
    let client = Client::new().unwrap(); // impls clone, is Send
    Interval::new_interval(poll_period)
        .zip(futures::stream::repeat(client))
        .map(|(_tick, client)| {
            // spawn stream that will do all of the necessary things
            // force the stream into a future so it may be spawned
            let work = client
                .activate_jobs(&activate_jobs_config)
                .and_then(|activated_job| {
                    // do work
                    Ok(())
                })
                .zip(futures::stream::repeat(client))
                .and_then(|(_, client)| {
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
