use crate::gateway;
use crate::gateway_grpc::Gateway;
use crate::worker::{JobConfig, WorkerConfig};
use futures::{Async, Stream};
use std::sync::Arc;

/// A future activates jobs and flattens them to a stream of gateway::ActivatedJob
pub struct ActivateJobs {
    s: Box<Stream<Item = gateway::ActivatedJob, Error = grpc::Error>>,
}

impl ActivateJobs {
    pub fn new(job_config: &JobConfig, worker_config: &WorkerConfig) -> Self {
        let stream = Self::create_activated_job_stream(job_config, worker_config);
        ActivateJobs { s: stream }
    }

    fn create_activate_jobs_request(
        job_config: &JobConfig,
        worker_config: &WorkerConfig,
    ) -> gateway::ActivateJobsRequest {
        let mut activate_jobs_request = gateway::ActivateJobsRequest::default();
        activate_jobs_request.set_amount(10); // TODO: make this configurable
        activate_jobs_request.set_timeout(job_config.timeout);
        activate_jobs_request.set_worker(worker_config.name.clone());
        activate_jobs_request.set_field_type(job_config.job_type.clone());
        activate_jobs_request
    }

    fn create_activate_jobs_response_stream(
        job_config: &JobConfig,
        worker_config: &WorkerConfig,
    ) -> Box<dyn Stream<Item = gateway::ActivateJobsResponse, Error = grpc::Error>> {
        let request = Self::create_activate_jobs_request(job_config, worker_config);
        let options = Default::default();
        let grpc_response: grpc::StreamingResponse<_> =
            worker_config.client.activate_jobs(options, request);
        let grpc_stream = grpc_response.drop_metadata();
        grpc_stream
    }

    /// flatten the batched up `ActivatedJob`s
    fn create_activated_job_stream(
        job_config: &JobConfig,
        worker_config: &WorkerConfig,
    ) -> Box<dyn Stream<Item = gateway::ActivatedJob, Error = grpc::Error>> {
        Box::new(
            Self::create_activate_jobs_response_stream(job_config, worker_config)
                .map(|r| futures::stream::iter_ok(r.jobs.into_iter()))
                .flatten(),
        )
    }
}

impl Stream for ActivateJobs {
    type Item = gateway::ActivatedJob;
    type Error = grpc::Error;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        self.s.poll()
    }
}

//use crate::gateway_grpc::Gateway;
//use crate::worker::job::Job;
//use crate::{gateway, Client};
//use futures::{Async, Future, Stream};
//use std::sync::{Arc, RwLock};
//
//struct ActivateJobs<F, J, S>
//where
//    S: Stream<Item = (i64, String), Error = grpc::Error>,
//    F: FnMut(i64, Option<String>) -> J,
//    J: Future<Item = Option<String>, Error = ()> + Send + 'static,
//{
//    max_active_jobs_count: usize,
//    active_jobs_count: Arc<RwLock<usize>>,
//    client: Arc<Client>,
//    stream: S,
//    job: Job<F, J>,
//}
//
//impl<F, J, S> ActivateJobs<F, J, S>
//where
//    S: Stream<Item = (i64, String), Error = grpc::Error>,
//    F: FnMut(i64, Option<String>) -> J,
//    J: Future<Item = Option<String>, Error = ()> + Send + 'static,
//{
//    pub fn new(max_active_jobs_count: usize,active_jobs_count: Arc<RwLock<usize>>,client: Arc<Client>) -> Self {
//        self.create_activate_jobs_stream(num_available_jobs)
//            .map(|activate_job_response| {
//                let jobs: Vec<_> = activate_job_response.jobs.into_iter().map(|activated_job: gateway::ActivatedJob| {
//                    let job_key = activated_job.key;
//                    let payload = activated_job.payload;
//                    (job_key, payload)
//                }).collect();
//                jobs
//            });
//
//        ActivateJobs {
//            max_active_jobs_count,
//            active_jobs_count,
//            client,
//        }
//    }
//
//    fn get_available_jobs_count(&self) -> usize {
//        let active_jobs_count = *self.active_jobs_count.read().unwrap();
//        if active_jobs_count > self.max_active_jobs_count {
//            return 0;
//        }
//        // Maximum allowed jobs running, so do nothing
//        if active_jobs_count == self.max_active_jobs_count {
//            return 0;
//        }
//        let available_jobs_count = self.max_active_jobs_count - active_jobs_count;
//        available_jobs_count
//    }
//
//    fn create_activate_jobs_request(
//        &self,
//        num_available_jobs: usize,
//    ) -> gateway::ActivateJobsRequest {
//        let mut activate_jobs_request = gateway::ActivateJobsRequest::default();
//        activate_jobs_request.set_amount(num_available_jobs as _);
//        activate_jobs_request.set_timeout(1000);
//        activate_jobs_request.set_worker("blah-worker".to_string());
//        activate_jobs_request.set_field_type("blah-worker-type".to_string());
//        activate_jobs_request
//    }
//
//    fn create_activate_jobs_stream(
//        &self,
//        num_available_jobs: usize,
//    ) -> Box<dyn Stream<Item = gateway::ActivateJobsResponse, Error = grpc::Error>> {
//        let request = self.create_activate_jobs_request(num_available_jobs);
//        let options = Default::default();
//        let grpc_response: grpc::StreamingResponse<_> =
//            self.client.gateway_client.activate_jobs(options, request);
//        let grpc_stream = grpc_response.drop_metadata();
//        grpc_stream
//    }
//}
//
//impl<F, J> Future for ActivateJobs<F, J>
//where
//    F: FnMut(i64, Option<String>) -> J,
//    J: Future<Item = Option<String>, Error = ()> + Send + 'static,
//{
//    type Item = ();
//    type Error = ();
//
//    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
//        let num_available_jobs = self.get_available_jobs_count();
//        if num_available_jobs == 0 {
//            return Ok(Async::Ready(())); // return anyway, no reason to keep polling this future
//        }
//
//        let client = self.client.clone();
//
//        self.create_activate_jobs_stream(num_available_jobs)
//            .map(|activate_job_response| {
//                let jobs: Vec<_> = activate_job_response.jobs.into_iter().map(|activated_job: gateway::ActivatedJob| {
//                    let job_key = activated_job.key;
//                    let payload = activated_job.payload;
//                    (job_key, payload)
//                }).collect();
//                jobs
//            });
//
//        // clone these for later
//        let lock = self.active_jobs_count.clone();
//        let mut do_work_future = self
//            .create_activate_jobs_stream(num_available_jobs)
//            .map_err(|_| ()) // TODO: implement an error type
//            .for_each(move |x| {
//                let activate_job_response: gateway::ActivateJobsResponse = x;
//                let new_active_jobs: Vec<gateway::ActivatedJob> =
//                    activate_job_response.jobs.into_vec();
//                let num_new_jobs = new_active_jobs.len();
//
//                // increment number of running jobs
//                let mut guard = lock.write().unwrap();
//                *guard += num_new_jobs;
//                drop(guard);
//
//                // spawn the new jobs
//                for job in new_active_jobs {
//                    let payload = if job.payload.len() > 0 {
//                        Some(job.payload)
//                    } else {
//                        None
//                    };
//                    let job_future = self
//                        .job
//                        .run_job(job.key, payload, client.clone())
//                        .complete_job(lock);
//                    //                        .map(|_| *lock.lock().unwrap() += 1);
//                    tokio::spawn(job_future);
//                }
//                futures::future::ok(())
//            });
//
//        do_work_future.poll()
//    }
//}
//

/*
    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        let num_available_jobs = self.get_available_jobs_count();
        if num_available_jobs == 0 {
            return Ok(Async::Ready(())); // return anyway, no reason to keep polling this future
        }

        let client = self.client.clone();

        // clone these for later
        let lock = self.active_jobs_count.clone();
        let mut do_work_future = self
            .create_activate_jobs_stream(num_available_jobs)
            .map_err(|_| ()) // TODO: implement an error type
            .for_each(move |x| {
                let activate_job_response: gateway::ActivateJobsResponse = x;
                let new_active_jobs: Vec<gateway::ActivatedJob> =
                    activate_job_response.jobs.into_vec();
                let num_new_jobs = new_active_jobs.len();

                // increment number of running jobs
                let mut guard = lock.write().unwrap();
                *guard += num_new_jobs;
                drop(guard);

                // spawn the new jobs
                for job in new_active_jobs {
                    let payload = if job.payload.len() > 0 {
                        Some(job.payload)
                    } else {
                        None
                    };
                    let job_future = self
                        .job
                        .run_job(job.key, payload, client.clone())
                        .complete_job(lock);
                    //                        .map(|_| *lock.lock().unwrap() += 1);
                    tokio::spawn(job_future);
                }
                futures::future::ok(())
            });

        do_work_future.poll()
    }
*/
