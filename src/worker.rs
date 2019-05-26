use crate::activate_jobs::ActivateJobsConfig;
use crate::client::Client;
use crate::complete_job::CompletedJobData;
use crate::gateway::ActivatedJob;

use futures::{Async, Future, IntoFuture, Stream};
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use tokio::timer::Interval;

#[derive(Clone)]
pub struct WorkerConfig {
    pub activate_jobs_config: ActivateJobsConfig,
    pub cancel_workflow_on_panic: bool,
}

pub struct WorkerFn<F, X>
where
    F: Fn(i64, String) -> X + Send + 'static,
    X: IntoFuture<Item = Option<String>, Error = ()> + Send,
    <X as IntoFuture>::Future: Send,
{
    f: F,
}

impl<F, X> WorkerFn<F, X>
where
    F: Fn(i64, String) -> X + Send + 'static,
    X: IntoFuture<Item = Option<String>, Error = ()> + Send,
    <X as IntoFuture>::Future: Send,
{
    pub fn new(f: F) -> Self {
        WorkerFn { f }
    }

    pub fn call(
        &self,
        job_key: i64,
        payload: String,
    ) -> impl Future<Item = (i64, Option<String>), Error = ()> + Send {
        futures::future::ok::<i64, ()>(job_key).join((self.f)(job_key, payload))
    }
}

pub fn work<F, X>(
    client: Arc<Client>,
    worker_config: WorkerConfig,
    f: F,
) -> impl Stream<Item = CompletedJobData, Error = ()> + Send
where
    F: Fn(i64, String) -> X + Send + 'static,
    X: IntoFuture<Item = Option<String>, Error = ()> + Send,
    <X as IntoFuture>::Future: Send,
{
    let work_fn = WorkerFn::<F, X>::new(f);
    client
        .activate_jobs(&worker_config.activate_jobs_config)
        .map_err(|_| panic!("grpc error"))
        .and_then(move |activated_job| work_fn.call(activated_job.key, activated_job.payload))
        .zip(futures::stream::repeat(client))
        .and_then(|((job_key, payload), client)| {
            let completed_job_data = CompletedJobData { job_key, payload };
            client.complete_job(completed_job_data).map_err(|_| ())
        })
}

///// A worker represents activated jobs that will be worked on and then completed
//pub struct Worker {
//    s: Box<dyn Stream<Item = CompletedJobData, Error = grpc::Error> + Send>,
//}
//
//impl Worker  {
//    pub fn new(client: Arc<Client>, worker_config: WorkerConfig, f: F) -> Self
//    where
//        F: Fn(i64, String) -> X + Send + 'static,
//        X: IntoFuture<Item = Option<String>, Error = ()>,
//    {
//        let worker_fn = WorkerFn::new(f);
//
//        let on_activated_job = move |activated_job: ActivatedJob| {
//            // do work
//            let job_key = activated_job.key;
//            let payload = activated_job.payload;
//            Ok((job_key, f(job_key, payload)))
//        };
//
//        let work = client
//            .activate_jobs(&worker_config.activate_jobs_config)
//            .and_then(on_activated_job)
//            .zip(futures::stream::repeat(client))
//            .and_then(|(data, client)| {
//                let completed_job_data = CompletedJobData {
//                    job_key: data.0,
//                    payload: data.1,
//                };
//                client.complete_job(completed_job_data)
//            });
//
//        Worker { s: Box::new(work), }
//    }
//}
//
//impl Stream for Worker {
//    type Item = CompletedJobData;
//    type Error = grpc::Error;
//
//    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
//
//        self.s.poll()
//    }
//}
//
///// Just like a `Worker` but will poll on an interval and repeatedly perform the worker tasks.
//pub struct WorkerInterval {
//    s: Box<dyn Stream<Item = CompletedJobData, Error = grpc::Error> + Send>,
//}
//
//impl WorkerInterval {
//    pub fn new<F>(client: Arc<Client>, worker_config: WorkerConfig, duration: Duration, f: F) -> Self
//        where
//            F: Fn(i64, String) -> Option<String> + Send + 'static, {
//        let s = Interval::new_interval(duration)
//            // make the worker available to future by zipping with it
//            .zip(futures::stream::repeat((client, worker_config, f)))
//            // on each tick, create the worker task
//            .and_then(|(_tick, (client, config, handler))| {
//                // spawn stream that will do all of the necessary things
//                // force the stream into a future so it may be spawned
//                client
//                    .worker(config, handler)
//            });
//
//        WorkerInterval {
//            s,
//        }
//
//    }
//}
//
//impl Stream for WorkerInterval {
//    type Item = CompletedJobData;
//    type Error = grpc::Error;
//
//    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
//        self.s.poll()
//    }
//}
