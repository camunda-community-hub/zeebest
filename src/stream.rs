use futures::{Stream, Poll, Async, Future};
use crate::{Client, gateway};
use futures::sync::mpsc::{channel, Receiver, Sender};

use crate::gateway_grpc::*;
use crate::gateway::{
    ActivateJobsRequest
};

use tokio::timer::Interval;
use std::time::Duration;
use futures::future::IntoFuture;

pub struct Work {
    sender: Sender<u32>,
}

impl Work {
    pub fn new(sender: Sender<u32>) -> Self {
        Work {
            sender,
        }
    }
}

impl Future for Work {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        Ok(Async::Ready(()))
    }
}

pub struct JobWorkerStream {
    max_active_jobs: usize,
    active_jobs: usize,
    client: Client,
    interval: Interval,
    sender: Sender<u32>,
}

impl JobWorkerStream {
    pub fn new(max_active_jobs: u8) -> Self {
        let (sender, receiver) = channel::<u32>(256);

        let receiver_stream = receiver.and_then(|x| {
            futures::future::ok(x)
        });

        JobWorkerStream {
            max_active_jobs: max_active_jobs as usize,
            active_jobs: 0,
            client: Client::new().unwrap(),
            interval: Interval::new_interval(Duration::from_secs(1)),
            sender,
        }
    }

    pub fn activate_new_jobs(&mut self) {
        let options = Default::default();
        let mut activate_jobs_request = ActivateJobsRequest::default();
        activate_jobs_request.set_amount(self.max_active_jobs as _);
        activate_jobs_request.set_timeout(1000);
        activate_jobs_request.set_worker("blah-worker".to_string());
        activate_jobs_request.set_field_type("blah-worker-type".to_string());
        let grpc_response: grpc::StreamingResponse<_> = self.client
            .gateway_client
            .activate_jobs(options, activate_jobs_request);

        let grpc_stream = grpc_response.drop_metadata();

//        interval_future.and_then();



        let results = grpc_stream.map_err(|_| ()).for_each(|x| {
            let activate_job_response: gateway::ActivateJobsResponse = x;

            let new_active_jobs: Vec<_> = activate_job_response.jobs.into_vec();

            let num_new_active_jobs = new_active_jobs.len();


            // calculate number of jobs to spawn
            let num_jobs_to_spawn = if self.active_jobs <= num_new_active_jobs {
                let num_jobs_to_spawn = self.active_jobs;
                self.active_jobs = 0;
                num_jobs_to_spawn
            }
            else {
                let num_jobs_to_spawn = num_new_active_jobs;
                self.active_jobs =  num_jobs_to_spawn - self.active_jobs;
                num_jobs_to_spawn
            };

            for _ in new_active_jobs.into_iter().take(num_jobs_to_spawn) {
                let work_future = Work::new(self.sender.clone()).into_future();
                tokio::spawn(work_future);
            }

            futures::future::ok(())
        });

//        future.and_then(|x| )d

    }
}

impl Stream for JobWorkerStream {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
//        try_ready!(
//            self.interval.poll().map_err(|_| ())
//        );
//
//        let options = Default::default();
//        let mut activate_jobs_request = ActivateJobsRequest::default();
//        activate_jobs_request.set_amount(self.max_active_jobs as _);
//        activate_jobs_request.set_timeout(1000);
//        activate_jobs_request.set_worker("blah-worker".to_string());
//        activate_jobs_request.set_field_type("blah-worker-type".to_string());
//        let grpc_response: grpc::StreamingResponse<_> = self.client
//            .gateway_client
//            .activate_jobs(options, activate_jobs_request);
//
//        let mut grpc_stream = grpc_response.drop_metadata();
//
//        let activate_jobs_response: Option<gateway::ActivateJobsResponse> = try_ready!(
//            grpc_stream.poll().map_err(|_| ())
//        );
//
//        if let None = activate_jobs_response {
//            return Ok(Async::NotReady)
//        }
        unimplemented!()

        // let jobs = activate_jobs_response.unwrap().get_jobs().into_iter().map(|job| job.)

//        Ok(Async::Ready(Some(())))

//        x.poll().map(|a| match a {
//            Async::Ready(Some(x)) => {
//            },
//            Async::Ready(None) => {
//
//            },
//            Async::NotReady => {
//
//            }
//        });

//        let results: Vec<_> = grpc_response
//            .wait_drop_metadata()
//            .as_mut()
//            .map(|r| {
//                r.map(|a| a.jobs.into_vec())
//                    .map_err(|e| ())
//            })
//            .collect();
//        results

//        self.client.gateway_client

//        unimplemented!()
    }
}
