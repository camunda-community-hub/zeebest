use crate::{
    ActivateJobs, ActivatedJob, ActivatedJobs, Client, CompleteJob, JobResult, PanicOption,
    WorkerConfig,
};
use futures::{Future, FutureExt, Stream, StreamExt};
use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, AtomicI32, Ordering};
use std::sync::Arc;

use tower::ServiceBuilder;
/*

#[derive(zeebest)]
#[zeebest(
    name = "rusty-worker",
    server = "127.0.0.1",
    port = 26500,
    )]
struct Worker {
#[zeebest(
        job_type = "",
        timeout = Duration::from_secs(2),

        about = "Deploy the workflow on the broker. You probably only need to do this once."
    )]
}

*/

pub struct ConcurrencyConfigBuilder {
    job_count: Option<Arc<AtomicI32>>,
    max_concurrent_jobs: Option<Arc<i32>>,
}

#[derive(Default)]
pub struct NewJobBuilder {
    job_handler: Option<JobHandler>,
    concurrent_config_builder: Option<ConcurrencyConfigBuilder>,
    client: Option<Client>,
    job_type: Option<String>,
    timeout: Option<i64>,
    panic_option: Option<PanicOption>,
}

impl NewJobBuilder {
    pub fn job_handler(&mut self, job_handler: JobHandler) { self.job_handler = Some(job_handler) }
    pub fn concurrency(&mut self, concurrency_config: ConcurrencyConfigBuilder) { self.concurrent_config_builder = Some(concurrency_config) }
    pub fn client(&mut self, client: Client) { self.client = Some(client) }
    pub fn job_type(&mut self, job_type: String) { self.job_type = Some(job_type) }
    pub fn timeout(&mut self, timeout: i64) { self.timeout = Some(timeout) }
    pub fn panic_option(&mut self, panic_option: PanicOption) { self.panic_option = Some(panic_option) }

    pub fn build(self) -> JobWorker {
//        Job::new()
        unimplemented!()
    }
}

pub struct NewJobWorkerBuilder {
    job_builders: Vec<NewJobBuilder>,
}

impl NewJobWorkerBuilder {
    pub fn job(&mut self, job: NewJobBuilder) {
        self.job_builders.push(job);
    }

    pub fn build(self) {
        for jb in self.job_builders {
            jb.build();
        }
    }
}

fn build_the_world() {
    let new_job_worker_builder = NewJobWorkerBuilder {
        job_builders: vec![
            NewJobBuilder::default(),
        ],
    };
}

type JobHandlerFn =
Box<dyn Fn(ActivatedJob) -> Pin<Box<dyn Future<Output = JobResult> + Send>> + Send + Sync>;

pub struct JobHandler {
    job_handler_fn: JobHandlerFn,
}

impl JobHandler {
    pub fn completed() -> Self {
        JobHandler {
            job_handler_fn: Box::new(|_| async { JobResult::Complete { variables: None } }.boxed()),
        }
    }
    pub fn ready<F>(job_handler: F, client: Client, worker_config: WorkerConfig) -> Self
    where
    F: Fn(ActivatedJob) -> JobResult + Sync + Send + 'static,
    {
        let fn1 = move |aj: ActivatedJob| {
            async {
                JobResult::Complete { variables: None }
            }.boxed()
        };

        JobHandler {
            job_handler_fn: Box::new(fn1),
        }


//        let jb = |aj| {
//            let jh = jh.clone();
//            async {
//                let job_result :JobResult = (jh.clone())(aj);
//                job_result
//            }.boxed()
//        };
//
//        JobHandler {
//            job_handler_fn: Box::new(jb)
//        }
    }
}

pub struct JobInternal {
    job_handler: JobHandlerFn,
    job_count: AtomicUsize,
    max_concurrent_jobs: usize,
    client: Client,
    worker_name: String,
    job_type: String,
    timeout: i64,
    panic_option: PanicOption,
}

impl JobInternal {
    pub async fn activate_and_process_jobs(self: Arc<Self>) {
        // TODO: insert back appropriate bounds checks and assert on invariants
        let current_job_count: usize = self.job_count.load(Ordering::SeqCst);
        let max_jobs_to_activate = self.max_concurrent_jobs - current_job_count;
        let mut activate_jobs = ActivateJobs::new(
            self.worker_name.clone(),
            self.job_type.clone(),
            self.timeout,
            0,
        );
        activate_jobs.max_jobs_to_activate = (self.max_concurrent_jobs - current_job_count) as _;
        let mut activate_jobs_stream = self.client.activate_jobs(activate_jobs);
        loop {
            match activate_jobs_stream.next().await {
                Some(Ok(ActivatedJobs { activated_jobs })) => {
                    let it = activated_jobs.into_iter().map(|activated_job| {
                        async {
                            self.job_count.fetch_add(1, Ordering::SeqCst);
                            let job_key: i64 = activated_job.key;
                            let retries = activated_job.retries;
                            match AssertUnwindSafe((self.job_handler)(activated_job))
                                .catch_unwind()
                                .await
                            {
                                Ok(JobResult::NoAction) => {}
                                Ok(JobResult::Complete { variables }) => {
                                    let complete_job = CompleteJob { job_key, variables };
                                    self.client.complete_job(complete_job).await.unwrap();
                                }
                                Ok(JobResult::Fail { .. }) => {
                                    self.client.fail_job(job_key, retries - 1).await.unwrap();
                                }
                                Err(_) => match self.panic_option {
                                    PanicOption::DoNothingOnPanic => {}
                                    PanicOption::FailJobOnPanic => {
                                        self.client.fail_job(job_key, retries - 1).await.unwrap();
                                    }
                                },
                            };
                            self.job_count.fetch_sub(1, Ordering::SeqCst);
                        }
                    });
                    futures::future::join_all(it).await;
                }
                Some(Err(e)) => {
                    println!("there was a problem activating jobs, {:?}", e);
                    break;
                }
                None => {
                    break;
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct JobWorker {
    job_internal: Arc<JobInternal>,
}

impl JobWorker {
    pub fn new<F>(
        worker: String,
        job_type: String,
        timeout: i64,
        max_amount: u16,
        panic_option: PanicOption,
        client: Client,
        job_handler: F,)-> Self
    where
        F: Fn(ActivatedJob) -> Pin<Box<dyn Future<Output = JobResult> + Send>>
            + Send
            + Sync
            + 'static,
    {
        let job_internal = Arc::new(JobInternal {
            job_handler: Box::new(job_handler),
            job_count: AtomicUsize::new(0),
            max_concurrent_jobs: max_amount as _,
            client,
            worker_name: worker,
            job_type,
            timeout,
            panic_option,
        });

        JobWorker {
            job_internal,
        }
    }

    pub async fn activate_and_process_jobs(self) {
        self.job_internal.activate_and_process_jobs().await;
    }
}

pub struct WorkerBuilder<S: Stream + Unpin> {
    interval: S,
    client: Client,
    handlers: HashMap<
        &'static str,
        (
            WorkerConfig,
            Box<
                dyn Fn(ActivatedJob) -> Pin<Box<dyn Future<Output = JobResult> + Send>>
                    + Send
                    + Sync,
            >,
        ),
    >,
}

impl<S: Stream + Unpin> WorkerBuilder<S> {
    pub fn add_job_handler<
        F: Fn(ActivatedJob) -> Pin<Box<dyn Future<Output = JobResult> + Send>> + Send + Sync + 'static,
    >(
        mut self,
        name: &'static str,
        wc: WorkerConfig,
        f: F,
    ) -> Self {
        self.handlers.insert(name, (wc, Box::new(f)));
        self
    }

    pub fn new_with_interval_and_client(interval: S, client: Client) -> Self {
        WorkerBuilder {
            interval,
            client,
            handlers: HashMap::new(),
        }
    }

    //    pub async fn into_future(self) {
    //        let client = self.client;
    //        let jobs: Vec<Arc<Job>> = self.handlers
    //            .into_iter()
    //            .map(|(_, (worker_config, job_handler))| {
    //                Arc::new(Job {
    //                    job_handler,
    //                    job_count: AtomicUsize::new(0),
    //                    max_concurrent_jobs: worker_config.max_concurrent_jobs as _,
    //                    client: client.clone(),
    //                    worker_name: worker_config.worker_name,
    //                    job_type: worker_config.job_type,
    //                    panic_option: worker_config.panic_option,
    //                    timeout: worker_config.timeout,
    //                })
    //            })
    //            .collect::<Vec<Arc<Job>>>();
    //        let mut interval = self.interval;
    //        while let Some(_) = interval.next().await {
    //            let it = jobs.iter().cloned().map(Job::activate_and_process_jobs);
    //            futures::future::join_all(it).await;
    //        };
    //    }
}
