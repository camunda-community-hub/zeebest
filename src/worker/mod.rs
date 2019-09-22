use crate::{ActivateJobs, ActivatedJob, ActivatedJobs, Client};
use futures::{Future, FutureExt, StreamExt};
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

mod job_handler;
mod job_client;

pub use job_handler::JobHandler;
pub use job_client::JobClient;
pub use job_client::Completer;

/// An option that describes what the job worker should do if if the job handler panics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PanicOption {
    FailJobOnPanic,
    DoNothingOnPanic,
}

/// A result that describes the output of a job.
#[derive(Clone, Debug, PartialEq)]
pub enum JobResult {
    Complete { variables: Option<String> },
    Fail { error_message: Option<String> },
    NoAction,
}

impl JobResult {
    pub fn into_result(self) -> Result<Option<String>, Option<String>> {
        match self {
            JobResult::Complete { variables } => Ok(variables),
            JobResult::Fail { error_message } => Err(error_message),
            JobResult::NoAction => Err(None),
        }
    }
}

pub struct JobInternal {
    job_handler: JobHandler,
    job_count: AtomicUsize,
    max_concurrent_jobs: usize,
    client: Client,
    job_client: JobClient,
    worker_name: String,
    job_type: String,
    timeout: i64,
    panic_option: PanicOption,
}

impl JobInternal {
    pub fn activate_and_process_jobs(self: Arc<Self>) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let current_job_count: usize = self.job_count.load(Ordering::SeqCst);
        // assert on an unreachable edge case
        assert!(
            current_job_count <= self.max_concurrent_jobs,
            "current number of jobs exceeds allowed number of running jobs"
        );
        let mut activate_jobs = ActivateJobs::new(
            self.worker_name.clone(),
            self.job_type.clone(),
            self.timeout,
            0,
        );
        activate_jobs.max_jobs_to_activate = (self.max_concurrent_jobs - current_job_count) as _;

        let activate_jobs_stream = self.client.activate_jobs(activate_jobs);

        let slf = self.clone();
        activate_jobs_stream.for_each_concurrent(None, move |result| {
            match result {
                Err(_e) => {
                    futures::future::ready(()).boxed()
                },
                Ok(ActivatedJobs { activated_jobs }) => {
                    let slf = slf.clone();
                    let job_count = activated_jobs.len();
                    slf.job_count.fetch_add(job_count, Ordering::SeqCst);
                    futures::stream::iter(activated_jobs).for_each_concurrent(None, move |aj| {
                        let slf = slf.clone();
                        slf.job_handler.process_job(aj.clone()).then(move |result| {
                            slf.job_count.fetch_sub(1, Ordering::SeqCst);
                            match result {
                                Err(_) => {
                                    match slf.panic_option {
                                        PanicOption::FailJobOnPanic => {
                                            slf.job_client.report_status(aj, JobResult::Fail { error_message: Some("worker panicked".to_string()) })
                                        },
                                        PanicOption::DoNothingOnPanic => {
                                            futures::future::ok(()).boxed()
                                        },
                                    }
                                },
                                Ok(job_result) => {
                                    slf.job_client.report_status(aj, job_result)
                                },
                            }
                        }).then(|_| futures::future::ready(()))
                    }).boxed()
                },
            }
        }).boxed()
    }
}

/// A worker will activate and process jobs with a job handler function.
/// The job worker has a maximum number of concurrent jobs that is controlled by an atomic,
/// consistent counter. When a job handler returns, the job worker will respond to the gateway
/// with the correct response (complete or fail). The job worker can be configured with a panic
/// option, so that it knows if to fail the job or do nothing if the handler panics.
///
/// The only method will poll for jobs, and if there are any, will start processing jobs. It will
/// only activate up to the maximum number of concurrent jobs allowed. When jobs complete, the counter
/// is updated.
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
        job_handler: F,
    ) -> Self
        where
            F: Fn(ActivatedJob) -> Pin<Box<dyn Future<Output = JobResult> + Send>>
            + Send
            + Sync
            + 'static,
    {
        let job_internal = Arc::new(JobInternal {
            job_client: JobClient::new(Completer::new(client.clone())),
            job_handler: JobHandler::new(Arc::new(job_handler)),
            job_count: AtomicUsize::new(0),
            max_concurrent_jobs: max_amount as _,
            client,
            worker_name: worker,
            job_type,
            timeout,
            panic_option,
        });

        JobWorker { job_internal }
    }

    pub fn activate_and_process_jobs(self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        self.job_internal.activate_and_process_jobs()
    }
}
