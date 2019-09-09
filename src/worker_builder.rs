use std::sync::Arc;
use std::panic::{AssertUnwindSafe};
use std::collections::HashMap;
use crate::{JobResult, ActivatedJob, ActivateJobs, PanicOption, CompleteJob, Client, WorkerConfig};
use futures::{Future, FutureExt, Stream, StreamExt};
use std::pin::Pin;

type JobHandlerFn = Arc<dyn Fn(ActivatedJob) -> Pin<Box<dyn Future<Output = JobResult> + Send>> + Send + Sync>;

pub struct WorkerBuilder<S: Stream + Unpin> {
    interval: S,
    client: Client,
    handlers: HashMap<&'static str, (WorkerConfig, Arc<dyn Fn(ActivatedJob) -> Pin<Box<dyn Future<Output = JobResult> + Send>> + Send + Sync>)>,
}

impl<S: Stream + Unpin> WorkerBuilder<S> {
    pub fn add_job_handler<F: Fn(ActivatedJob) -> Pin<Box<dyn Future<Output = JobResult> + Send>> + Send + Sync + 'static>(mut self, name: &'static str, wc: WorkerConfig, f: F) -> Self {
        self.handlers.insert(name,  (wc, Arc::new(f)));
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
        let v: Vec<_> = self.handlers.into_iter().map(|(n, (wc, f))| (n,wc,f, client.clone())).collect();
        let mut interval = self.interval;
        while let Some(_) = interval.next().await {
            let i = v.iter().cloned().map(|(_n, wc, f, client)| {
                async move {
                    let activate_jobs = ActivateJobs::new(wc.worker_name.clone(), wc.job_type.clone(), wc.timeout, wc.max_jobs_to_activate);
                    let mut activate_jobs_stream = client.activate_jobs(activate_jobs);
                    loop {
                        match activate_jobs_stream.next().await {
                            Some(Ok(j)) => {
                                let it = j.activated_jobs.into_iter().map(|aj| {
                                    async {
                                        Self::process_activated_job(f.clone(), client.clone(), wc.panic_option, aj)
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
            });
            futures::future::join_all(i).await;
        }
    }

    async fn process_activated_job(job_handler: JobHandlerFn, client: Client, panic_option: PanicOption, activated_job: ActivatedJob) {
        let job_key: i64 = activated_job.key;
        let retries = activated_job.retries;
        match AssertUnwindSafe(job_handler(activated_job)).catch_unwind().await {
            Ok(JobResult::NoAction) => {},
            Ok(JobResult::Complete {variables}) => {
                println!("complete job");
                let complete_job = CompleteJob { job_key, variables };
                client.complete_job(complete_job).await.unwrap();
            },
            Ok(JobResult::Fail {..}) => {
                client.fail_job(job_key, retries - 1).await.unwrap();
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
    }
}
