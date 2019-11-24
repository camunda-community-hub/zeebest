




mod job_client;
mod job_handler;


//pub use job_client::JobClient;
//pub use job_client::Reporter;
//pub use job_handler::JobHandler;
//
///// An option that describes what the job worker should do if if the job handler panics.
//#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
//pub enum PanicOption {
//    FailJobOnPanic,
//    DoNothingOnPanic,
//}
//
///// A result that describes the output of a job.
//#[derive(Clone, Debug, PartialEq)]
//pub enum JobResult {
//    Complete { variables: Option<String> },
//    Fail { error_message: Option<String> },
//    NoAction,
//}
//
//impl JobResult {
//    pub fn into_result(self) -> Result<Option<String>, Option<String>> {
//        match self {
//            JobResult::Complete { variables } => Ok(variables),
//            JobResult::Fail { error_message } => Err(error_message),
//            JobResult::NoAction => Err(None),
//        }
//    }
//}
//
//pub struct JobInternal {
//    job_handler: JobHandler,
//    job_count: AtomicUsize,
//    max_concurrent_jobs: usize,
//    client: RwLock<Client>,
//    job_client: RwLock<JobClient>,
//    worker_name: String,
//    job_type: String,
//    timeout: i64,
//    panic_option: PanicOption,
//}
//
//impl JobInternal {
//    pub fn activate_and_process_jobs(self: Arc<Self>) -> Pin<Box<dyn Future<Output = ()> + Send>> {
//        let current_job_count: usize = self.job_count.load(Ordering::SeqCst);
//        // assert on an unreachable edge case
//        assert!(
//            current_job_count <= self.max_concurrent_jobs,
//            "current number of jobs exceeds allowed number of running jobs"
//        );
//        let mut activate_jobs = ActivateJobs::new(
//            self.worker_name.clone(),
//            self.job_type.clone(),
//            self.timeout,
//            0,
//        );
//        activate_jobs.max_jobs_to_activate = (self.max_concurrent_jobs - current_job_count) as _;
//
//        //        let activate_jobs_stream = {
//        //            let mut client = self.client.write().unwrap();
//        //            client.activate_jobs(activate_jobs)
//        //        };
//
//        futures::future::ready(()).boxed()
//        //
//        //        let slf = self.clone();
//        //        activate_jobs_stream
//        //            .for_each_concurrent(None, move |result| match result {
//        //                Err(_e) => futures::future::ready(()).boxed(),
//        //                Ok(ActivatedJobs { activated_jobs }) => {
//        //                    let slf = slf.clone();
//        //                    let job_count = activated_jobs.len();
//        //                    slf.job_count.fetch_add(job_count, Ordering::SeqCst);
//        //                    futures::stream::iter(activated_jobs)
//        //                        .for_each_concurrent(None, move |aj| {
//        //                            let slf = slf.clone();
//        //                            slf.job_handler
//        //                                .process_job(aj.clone())
//        //                                .then(move |result| {
//        //                                    slf.job_count.fetch_sub(1, Ordering::SeqCst);
//        //                                    match result {
//        //                                        Err(_) => match slf.panic_option {
//        //                                            PanicOption::FailJobOnPanic => {
//        //                                                let mut job_client = slf.job_client.write().unwrap();
//        //                                                job_client.report_status(
//        //                                                    aj,
//        //                                                    JobResult::Fail {
//        //                                                        error_message: Some(
//        //                                                            "worker panicked".to_string(),
//        //                                                        ),
//        //                                                    },
//        //                                                )
//        //                                            }
//        //                                            PanicOption::DoNothingOnPanic => {
//        //                                                futures::future::ok(()).boxed()
//        //                                            }
//        //                                        },
//        //                                        Ok(job_result) => {
//        //                                            let mut job_client = slf.job_client.write().unwrap();
//        //                                            job_client.report_status(aj, job_result)
//        //                                        }
//        //                                        _ => unimplemented!()
//        //                                    }
//        //                                })
//        //                                .then(|_| futures::future::ready(()))
//        //                        })
//        //                        .boxed()
//        //                }
//        //            })
//        //            .boxed()
//    }
//}
//
///// A worker will activate zeebe jobs and us the job handler to process those jobs concurrently.
//#[derive(Clone)]
//pub struct JobWorker {
//    job_internal: Arc<JobInternal>,
//}
//
//impl JobWorker {
//    pub fn new<F>(
//        _worker: String,
//        _job_type: String,
//        _timeout: i64,
//        _max_amount: u16,
//        _panic_option: PanicOption,
//        _client: Client,
//        _job_handler: F,
//    ) -> Self
//    where
//        F: Fn(ActivatedJob) -> Pin<Box<dyn Future<Output = JobResult> + Send>>
//            + Send
//            + Sync
//            + 'static,
//    {
//        unimplemented!()
//
//        //        let job_internal = Arc::new(JobInternal {
//        //            job_client: RwLock::new(JobClient::new(Reporter::new(client.clone()))),
//        //            job_handler: JobHandler::new(Arc::new(job_handler)),
//        //            job_count: AtomicUsize::new(0),
//        //            max_concurrent_jobs: max_amount as _,
//        //            client: RwLock::new(client),
//        //            worker_name: worker,
//        //            job_type,
//        //            timeout,
//        //            panic_option,
//        //        });
//        //
//        //        JobWorker { job_internal }
//    }
//
//    /// Activates a batch of jobs and processes each job with the job handler. Will not activate
//    /// more jobs that `max_concurrent_jobs - current_job_count`.
//    pub fn activate_and_process_jobs(self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
//        self.job_internal.activate_and_process_jobs()
//    }
//}
