use std::sync::Arc;
use std::panic::UnwindSafe;
use std::collections::HashMap;
use crate::{ActivatedJobs, JobResult, ActivatedJob, ActivateJobs, PanicOption, CompleteJob, Client, WorkerConfig};
use futures::{Future, FutureExt, Stream, StreamExt};

pub struct WorkerBuilder <I, Fut1>
    where
        I: Stream + Unpin,
        Fut1: Future<Output = JobResult> + Send + UnwindSafe + 'static,
//        F: Fn(ActivatedJob) -> Fut1 + Send + Sync + 'static,
{
    interval: I,
    workers: HashMap<String, (WorkerConfig, Arc<dyn Fn(ActivatedJob) -> Fut1 + Send + Sync>)>
}

impl<I, Fut1> WorkerBuilder<I, Fut1>
    where
        I: Stream + Unpin,
        Fut1: Future<Output = JobResult> + Send + UnwindSafe + 'static,
{
    pub fn with_interval(interval: I) -> Self {
        WorkerBuilder {
            interval,
            workers: HashMap::new(),
        }
    }

    pub fn set_worker<F: Fn(ActivatedJob) -> Fut1 + Send + Sync + 'static>(mut self, wc: WorkerConfig, f: F) -> Self {
        let job_type = wc.job_type.clone();
        self.workers.insert(job_type, (wc, Arc::new(f)));
        self
    }

    pub async fn build(self, client: Arc<Client>) -> () {
        let mut interval = self.interval;
        let job_handlers: Vec<(String, WorkerConfig, Arc<dyn Fn(ActivatedJob) -> Fut1 + Send + Sync>)> = self.workers.into_iter().map(|(jt,(wc, f))| (jt, wc, f)).collect();
        while let Some(_) = interval.next().await {
            let activate_jobs = ActivateJobs::new("the_worker", "the_job_type", 10000, 10);
            for jh in job_handlers.iter() {
                let (job_type, wc, f): &(String, WorkerConfig, Arc<dyn Fn(ActivatedJob) -> Fut1 + Send + Sync>) = jh;
//                let client = self.client.clone().unwrap();
                runtime::spawn(async move {
                    let activate_jobs = ActivateJobs::new(wc.worker_name.clone(), job_type.to_string(), wc.timeout, wc.max_jobs_to_activate);
                    let panic_option = wc.panic_option;
                    let mut activate_jobs_stream = client.activate_jobs(activate_jobs);
                    while let Some(Ok(activated_jobs)) = activate_jobs_stream.next().await {
                        for aj in activated_jobs.activated_jobs.into_iter() {
                            let f = f.clone();
                            let klient = client.clone();
                            let job_key = aj.key;
                            let retries = aj.retries;
                            runtime::spawn(async move {
                                match f(aj).catch_unwind().await {
                                    Ok(JobResult::NoAction) => {},
                                    Ok(JobResult::Complete {variables}) => {
                                        let complete_job = CompleteJob { job_key, variables };
                                        klient.complete_job(complete_job).await.unwrap();
                                    },
                                    Ok(JobResult::Fail {error_message}) => {
                                        klient.fail_job(job_key, retries - 1).await.unwrap();
                                    }
                                    Err(_) => {
                                        match panic_option {
                                            PanicOption::DoNothingOnPanic => {
                                            },
                                            PanicOption::FailJobOnPanic => {
                                            }
                                        }
                                    },
                                };
                            });
                        };
                    }
                });
            }
        }
        ()
    }
}