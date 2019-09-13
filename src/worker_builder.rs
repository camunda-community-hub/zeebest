use std::sync::{Arc, RwLock};
use std::panic::{AssertUnwindSafe};
use std::collections::HashMap;
use crate::{JobResult, ActivatedJob, ActivateJobs, PanicOption, CompleteJob, Client, WorkerConfig, ActivatedJobs};
use futures::{Future, FutureExt, Stream, StreamExt, Poll};
use std::pin::Pin;
use std::sync::atomic::{AtomicU16, Ordering};
use futures::task::Context;

type JobHandlerFn = Arc<dyn Fn(ActivatedJob) -> Pin<Box<dyn Future<Output = JobResult> + Send>> + Send + Sync>;

type JobHndlrFn = Box<dyn Fn(ActivatedJob) -> Pin<Box<dyn Future<Output = JobResult> + Send>> + Send + Sync>;

struct Job {
    job_handler: JobHndlrFn,
    job_count: RwLock<AtomicU16>,
    client: Client,
    activate_jobs: ActivateJobs,
    panic_option: PanicOption,
}

impl Job {
    async fn activate_and_process_jobs_for_unit(self: Arc<Self>) {
        let mut activate_jobs_stream = self.client.activate_jobs(self.activate_jobs.clone());
        loop {
            match activate_jobs_stream.next().await {
                Some(Ok(ActivatedJobs { activated_jobs })) => {
                    let it = activated_jobs.into_iter().map(|aj| {
                        async {
                            self.clone().process_activated_job(aj)
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

    async fn process_activated_job(self: Arc<Self>, activated_job: ActivatedJob) {
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
                    }
                }
            },
        };
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
                    job_count: RwLock::new(AtomicU16::new(0)),
                    client: client.clone(),
                    activate_jobs: ActivateJobs::new(worker_config.worker_name, worker_config.job_type, worker_config.timeout, worker_config.max_jobs_to_activate),
                    panic_option: worker_config.panic_option,
                })
            })
            .collect::<Vec<Arc<Job>>>();
        let mut interval = self.interval;
        while let Some(_) = interval.next().await {
            futures::future::join_all(jobs.iter().cloned().map(Job::activate_and_process_jobs_for_unit)).await;
        };
    }

    async fn activate_and_process_jobs_for_unit(unit: &Unit, current_job_count: Arc<AtomicU16>) {
        let job_handler = unit.job_handler.clone();
        let worker_name = unit.worker_config.worker_name.clone();
        let job_type = unit.worker_config.job_type.clone();
        let timeout = unit.worker_config.timeout;
        let max_jobs_to_activate = unit.worker_config.max_jobs_to_activate;
        let activate_jobs = ActivateJobs::new(worker_name, job_type, timeout, max_jobs_to_activate);
        let panic_option = unit.worker_config.panic_option;
        let mut activate_jobs_stream = unit.client.activate_jobs(activate_jobs);
        loop {
            match activate_jobs_stream.next().await {
                Some(Ok(ActivatedJobs { activated_jobs })) => {
                    let it = activated_jobs.into_iter().map(|aj| {
                        async {
                            Self::process_activated_job(unit, aj)
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

    async fn process_activated_job(unit: &Unit, activated_job: ActivatedJob) {
        let job_key: i64 = activated_job.key;
        let retries = activated_job.retries;
        match AssertUnwindSafe((unit.job_handler)(activated_job)).catch_unwind().await {
            Ok(JobResult::NoAction) => {},
            Ok(JobResult::Complete {variables}) => {
                println!("complete job");
                let complete_job = CompleteJob { job_key, variables };
                unit.client.complete_job(complete_job).await.unwrap();
            },
            Ok(JobResult::Fail {..}) => {
                unit.client.fail_job(job_key, retries - 1).await.unwrap();
            }
            Err(_) => {
                match unit.worker_config.panic_option {
                    PanicOption::DoNothingOnPanic => {
                    },
                    PanicOption::FailJobOnPanic => {
                    }
                }
            },
        };
    }
}

struct Unit {
    job_handler: JobHandlerFn,
    client: Client,
    worker_config: WorkerConfig,
}
