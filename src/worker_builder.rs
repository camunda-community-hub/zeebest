use std::sync::Arc;
use std::panic::{AssertUnwindSafe};
use std::collections::HashMap;
use crate::{JobResult, ActivatedJob, ActivateJobs, PanicOption, CompleteJob, Client, WorkerConfig, ActivatedJobs};
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
        let units: Vec<_> = self.handlers.into_iter().map(|(_, (worker_config, job_handler))| Unit { worker_config, job_handler, client: client.clone() }).collect();
        let mut interval = self.interval;
        while let Some(_) = interval.next().await {
            let work_units_iterator = units.iter().map(Self::activate_and_process_jobs_for_unit);
            futures::future::join_all(work_units_iterator).await;
        }
    }

    async fn activate_and_process_jobs_for_unit(unit: &Unit) {
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
