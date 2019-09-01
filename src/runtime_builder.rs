use futures::{Stream, Future, FutureExt, StreamExt};
use crate::{ActivatedJobs, JobResult, ActivatedJob, ActivateJobs, PanicOption, CompleteJob, Client, WorkerConfig};
use std::panic::UnwindSafe;
use std::sync::Arc;
use std::collections::HashMap;

pub struct RuntimeBuilder<I, Fut1, F>
where
    I: Stream + Unpin,
    Fut1: Future<Output = JobResult> + Send + UnwindSafe + 'static,
    F: Fn(ActivatedJob) -> Fut1 + Send + Sync + 'static,
{
    client: Option<Client>,
    interval: Option<I>,
    job_handlers: HashMap<WorkerConfig, Arc<F>>
}

impl<I, Fut1, F> RuntimeBuilder<I, Fut1, F>
    where
        I: Stream + Unpin,
        Fut1: Future<Output = JobResult> + Send + UnwindSafe + 'static,
        F: Fn(ActivatedJob) -> Fut1 + Send + Sync + 'static,
{
    pub fn new() -> Self {
        RuntimeBuilder {
            client: None,
            interval: None,
            job_handlers: Default::default(),
        }
    }

    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    pub fn interval(mut self, i: I) -> Self {
        self.interval = Some(i);
        self
    }

    pub fn worker(mut self, wc: WorkerConfig, f: F) -> Self {
        let mp = &mut self.job_handlers;
        mp.insert(wc, Arc::new(f));
        self
    }

    pub async fn build(self) -> () {
        let mut interval = self.interval.unwrap();
        let job_handlers = self.job_handlers;
        interval.next().await.unwrap();
        while let Some(_) = interval.next().await {
            let activate_jobs = ActivateJobs::new("the_worker", "the_job_type", 10000, 10);
            for (ajs, f) in job_handlers.clone() {
                let client = self.client.clone().unwrap();
                runtime::spawn(async move {
                    let activate_jobs = ActivateJobs::new(ajs.worker, ajs.job_type, ajs.timeout, ajs.max_jobs_to_activate);
                    let panic_option = ajs.panic_option;
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