use std::sync::{Arc};
use std::panic::{AssertUnwindSafe};
use std::collections::HashMap;
use crate::{JobResult, ActivatedJob, ActivateJobs, PanicOption, CompleteJob, Client, WorkerConfig, ActivatedJobs};
use futures::{Future, FutureExt, Stream, StreamExt};
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};

type JobHandlerFn = Box<dyn Fn(ActivatedJob) -> Pin<Box<dyn Future<Output = JobResult> + Send>> + Send + Sync>;

struct Job {
    job_handler: JobHandlerFn,
    job_count: AtomicUsize,
    max_job_count: usize,
    client: Client,
    activate_jobs: ActivateJobs,
    panic_option: PanicOption,
}

impl Job {
    async fn activate_and_process_jobs(self: Arc<Self>) {
        // TODO: insert back appropriate bounds checks and assert on invariants
        let current_job_count: usize = self.job_count.load(Ordering::SeqCst);
        let mut activate_jobs = self.activate_jobs.clone();
        activate_jobs.max_jobs_to_activate = (self.max_job_count - current_job_count) as _;
        let mut activate_jobs_stream = self.client.activate_jobs(activate_jobs);
        loop {
            match activate_jobs_stream.next().await {
                Some(Ok(ActivatedJobs { activated_jobs })) => {
                    let it = activated_jobs
                        .into_iter()
                        .map(|activated_job| {
                            async {
                                self.job_count.fetch_add(1, Ordering::SeqCst);
                                let job_key: i64 = activated_job.key;
                                let retries = activated_job.retries;
                                match AssertUnwindSafe((self.job_handler)(activated_job)).catch_unwind().await {
                                    Ok(JobResult::NoAction) => {},
                                    Ok(JobResult::Complete {variables}) => {
                                        println!("complete job");
                                        let complete_job = CompleteJob { job_key, variables };
                                        self.client.complete_job(complete_job).await.unwrap();
                                    },
                                    Ok(JobResult::Fail {..}) => {
                                        self.client.fail_job(job_key, retries - 1).await.unwrap();
                                    }
                                    Err(_) => {
                                        match self.panic_option {
                                            PanicOption::DoNothingOnPanic => {
                                            },
                                            PanicOption::FailJobOnPanic => {
                                                self.client.fail_job(job_key, retries - 1).await.unwrap();
                                            }
                                        }
                                    },
                                };
                                self.job_count.fetch_sub(1, Ordering::SeqCst);
                            }
                        });
                    futures::future::join_all(it).await;
                },
                Some(Err(e)) => {
                    println!("there was a problem activating jobs, {:?}", e);
                    break;
                },
                None => {
                    break;
                }
            }
        }
    }
}

pub struct WorkerBuilder<S: Stream + Unpin> {
    interval: S,
    client: Client,
    handlers: HashMap<&'static str, (WorkerConfig, Box<dyn Fn(ActivatedJob) -> Pin<Box<dyn Future<Output = JobResult> + Send>> + Send + Sync>)>,
}

impl<S: Stream + Unpin> WorkerBuilder<S> {
    pub fn add_job_handler<F: Fn(ActivatedJob) -> Pin<Box<dyn Future<Output = JobResult> + Send>> + Send + Sync + 'static>(mut self, name: &'static str, wc: WorkerConfig, f: F) -> Self {
        self.handlers.insert(name,  (wc, Box::new(f)));
        self
    }

    pub fn new_with_interval_and_client(interval: S, client: Client) -> Self {
        WorkerBuilder {
            interval,
            client,
            handlers: HashMap::new(),
        }
    }

    pub async fn into_future(self) {
        let client = self.client;
        let jobs = self.handlers
            .into_iter()
            .map(|(_, (worker_config, job_handler))| {
                Arc::new(Job {
                    job_handler,
                    job_count: AtomicUsize::new(0),
                    max_job_count: worker_config.max_jobs_to_activate as _,
                    client: client.clone(),
                    activate_jobs: ActivateJobs::new(worker_config.worker_name, worker_config.job_type, worker_config.timeout, worker_config.max_jobs_to_activate),
                    panic_option: worker_config.panic_option,
                })
            })
            .collect::<Vec<Arc<Job>>>();
        let mut interval = self.interval;
        while let Some(_) = interval.next().await {
            let it = jobs.iter().cloned().map(Job::activate_and_process_jobs);
            futures::future::join_all(it).await;
        };
    }
}
