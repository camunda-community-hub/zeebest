//use crate::activate_and_process_jobs::activate_and_process_jobs;
//use crate::activate_jobs::ActivateJobsConfig;
//use crate::gateway;
//use crate::gateway_grpc;
//use crate::PanicOption;
//use crate::{Error, JobError, JobFnLike, JobFn, JobResponse, JobResult};
//use futures::stream::Stream;
//use std::sync::Arc;
//
//use futures::{IntoFuture};
//
//pub struct Worker {
//    client: Arc<gateway_grpc::GatewayClient>,
//    name: String,
//    timeout: i64,
//    amount: i32,
//    panic_option: PanicOption,
//    jobs: Option<Box<Stream<Item = JobResult, Error = Error>>>,
//    jobs_list: Vec<Arc<JobFnLike>>,
//    jobs_factory: Option<Box<Fn() -> Stream<Item = JobResult, Error = Error>>>,
//}
//
//impl Worker {
//    pub fn new<N: Into<String>>(name: N, client: Arc<gateway_grpc::GatewayClient>) -> Self {
//        Worker {
//            client,
//            name: name.into(),
//            timeout: 1000,
//            amount: 32,
//            panic_option: PanicOption::DoNothingOnPanic,
//            jobs: None,
//            jobs_factory: None,
//            jobs_list: vec![],
//        }
//    }
//
//    pub fn default_timeout(mut self, timeout: i64) -> Self {
//        self.timeout = timeout;
//        self
//    }
//
//    pub fn default_amount(mut self, amount: i32) -> Self {
//        self.amount = amount;
//        self
//    }
//
//    pub fn default_panic_option(mut self, panic_option: PanicOption) -> Self {
//        self.panic_option = panic_option;
//        self
//    }
//
//    pub fn job<F, X>(mut self, mut job_fn: JobFn<F, X>) -> Worker
//    where
//        F: Fn(gateway::ActivatedJob) -> X + Send + 'static,
//        X: IntoFuture<Item = JobResponse, Error = JobError> + 'static,
//        <X as futures::future::IntoFuture>::Future: std::panic::UnwindSafe,
//    {
//        // store the job for later
//        self.jobs_list.push(Arc::new(job_fn));
//
////        let activate_jobs_config = ActivateJobsConfig {
////            worker: self.name.clone(),
////            job_type: job_fn.job_type.clone(),
////            timeout: job_fn.timeout.take().unwrap_or(self.timeout),
////            amount: job_fn.amount.take().unwrap_or(self.amount),
////        };
////
////        let s = activate_and_process_jobs(
////            self.client.clone(),
////            activate_jobs_config,
////            job_fn
////                .panic_option
////                .take()
////                .unwrap_or(self.panic_option.clone()),
////            job_fn,
////        );
////
////        self.jobs = Some(match self.jobs {
////            Some(previous_s) => Box::new(previous_s.select(s)),
////            None => Box::new(s),
////        });
//
//        self
//    }
//
////    pub fn into_job_stream(self) -> impl Stream<Item = JobResult, Error = Error> {
////        self.jobs.unwrap_or(Box::new(futures::stream::empty()))
////    }
//
//    pub fn process_jobs(&self) -> Box<dyn Stream<Item = JobResult, Error = Error>> {
//        let global_timeout = self.timeout;
//        let global_amount = self.amount;
//        let global_panic_option = self.panic_option;
//        let worker = self.name.clone();
//
//        let job_stream: Option<Box<dyn Stream<Item = JobResult, Error = Error>>> = self.jobs_list.clone().into_iter().map(move |job_fn| {
//            let activate_jobs_config = ActivateJobsConfig {
//                worker: self.name.clone(),
//                job_type: job_fn.job_type(),
//                timeout: job_fn.timeout().unwrap_or(global_timeout),
//                amount: job_fn.amount().unwrap_or(global_amount),
//            };
//
//            let s = activate_and_process_jobs(
//                self.client.clone(),
//                activate_jobs_config,
//                job_fn
//                    .panic_option()
//                    .unwrap_or(global_panic_option),
//                job_fn,
//            );
//
//            Box::new(s)
//        })
//        .fold(None, |acc, next| {
//            Some(match acc {
//                Some(previous) => Box::new(previous.select(next)),
//                None => Box::new(next),
//            })
//        });
//
//        job_stream.unwrap_or(Box::new(futures::stream::empty()))
//    }
//}
