use tower::Service;
use futures::task::Context;
use futures::{Poll, Future, FutureExt, StreamExt};
use std::sync::{RwLock, Arc};
use std::pin::Pin;

enum Job {
    Activated,
    Done,
}

struct ActivatorSvc {
    jobs: Arc<RwLock<Vec<Job>>>,
    job_type: String,
    max_jobs_to_activate: Arc<RwLock<i32>>,
    client: zeebest::Client,
}

impl ActivatorSvc {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(RwLock::new(vec![])),
            job_type: String::new(),
            max_jobs_to_activate: Arc::new(RwLock::new(10)),
            client: zeebest::Client::new("127.0.0.1", 26500).unwrap()
        }
    }

    pub fn get_job_count(&self) -> usize {
        self.jobs.read().unwrap().len()
    }

    pub fn get_max_jobs_to_activate(&self) -> usize {
        *self.max_jobs_to_activate.read().unwrap() as usize
    }

    pub fn add_jobs(&self, jobs: Vec<Job>) {
        let mut current_jobs = self.jobs.write().unwrap();
        for job in jobs {
            current_jobs.push(job);
        }
    }
}

impl Service<()> for ActivatorSvc {
    type Response = ();
    type Error = Box<dyn std::error::Error + Send>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: ()) -> Self::Future {
        let current_job_count = self.get_job_count();
        let max_jobs_to_activate = self.get_max_jobs_to_activate();
        if current_job_count >= max_jobs_to_activate {
            futures::future::ok(()).boxed()
        }
        else {
            let request_job_count_max = (max_jobs_to_activate - current_job_count) as i32;
            let activate_jobs_request = zeebest::ActivateJobs::new("tower-worker", "foo_job_type", 3000, request_job_count_max);
            let mut stream = self.client.activate_jobs(activate_jobs_request);
            let current_jobs = self.jobs.clone();
            let x = async move {
                let current_jobs = current_jobs;
                let fut = stream.for_each_concurrent(None,|ajs| {
                    let ajs = ajs.unwrap().activated_jobs;
                    let jobs = vec![];
                    let cjs = current_jobs.clone();
                    let mut cjs = cjs.write().unwrap();
                    for job in jobs {
                        cjs.push(job);
                    }
                    futures::future::ready(())
                }).await;
                Ok(())
            };
            x.boxed()
        }
    }
}

struct ActivatorConfig;

struct MakeActivatorSvc;
impl Service<ActivatorConfig> for MakeActivatorSvc {
    type Response = ActivatorSvc;
    type Error = Box<dyn std::error::Error + 'static>;
    type Future = Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Unpin>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: ActivatorConfig) -> Self::Future {
        Box::new(futures::future::ok(ActivatorSvc::new()))
    }
}

fn main() {

}