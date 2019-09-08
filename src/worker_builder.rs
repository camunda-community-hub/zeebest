use std::sync::Arc;
use std::panic::{UnwindSafe, AssertUnwindSafe};
use std::collections::HashMap;
use crate::{ActivatedJobs, JobResult, ActivatedJob, ActivateJobs, PanicOption, CompleteJob, Client, WorkerConfig};
use futures::{Future, FutureExt, Stream, StreamExt};
use std::pin::Pin;

pub struct WorkerBuilder<S: Stream + Unpin> {
    interval: S,
    client: Arc<Client>,
    handlers: HashMap<&'static str, (WorkerConfig, Arc<dyn Fn(ActivatedJob) -> Pin<Box<dyn Future<Output = JobResult> + Send>> + Send + Sync>)>,
}

impl<S: Stream + Unpin> WorkerBuilder<S> {
    pub fn add_job_handler<F: Fn(ActivatedJob) -> Pin<Box<dyn Future<Output = JobResult> + Send>> + Send + Sync + 'static>(mut self, name: &'static str, wc: WorkerConfig, f: F) -> Self {
        self.handlers.insert(name,  (wc, Arc::new(f)));
        self
    }
    pub fn new_with_interval_and_client(interval: S, client: Arc<Client>) -> Self {
        WorkerBuilder {
            interval,
            client,
            handlers: HashMap::new(),
        }
    }

    pub async fn into_future(mut self) {
        let client = self.client;
        let v: Vec<_> = self.handlers.into_iter().map(|(n, (wc, f))| (n,wc,f, client.clone())).collect();
        let mut interval = self.interval;
        while let Some(_) = interval.next().await {
            let i = v.iter().cloned().map(|(n, wc, f, client)| {
                async move {
                    let activate_jobs = ActivateJobs::new(wc.worker_name.clone(), wc.job_type.clone(), wc.timeout, wc.max_jobs_to_activate);
                    let mut activate_jobs_stream = client.activate_jobs(activate_jobs);

                    loop {
                        match activate_jobs_stream.next().await {
                            Some(Ok(j)) => {
                                let it = j.activated_jobs.into_iter().map(|aj| {
                                    async {
                                        let f = f.clone();
                                        let client = client.clone();
                                        let job_key: i64 = aj.key;
                                        let retries = aj.retries;
                                        let panic_option = PanicOption::FailJobOnPanic;
                                        match AssertUnwindSafe(f(aj)).catch_unwind().await {
                                            Ok(JobResult::NoAction) => {},
                                            Ok(JobResult::Complete {variables}) => {
                                                println!("complete job");
                                                let complete_job = CompleteJob { job_key, variables };
                                                client.complete_job(complete_job).await.unwrap();
                                            },
                                            Ok(JobResult::Fail {error_message}) => {
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

//                    while let Some(Ok(j)) = activate_jobs_stream.next().await {
//                        let it = j.activated_jobs.into_iter().map(|aj| {
//                            async {
//                                let f = f.clone();
//                                let client = client.clone();
//                                let job_key: i64 = aj.key;
//                                let retries = aj.retries;
//                                let panic_option = PanicOption::FailJobOnPanic;
//                                match AssertUnwindSafe(f(aj)).catch_unwind().await {
//                                    Ok(JobResult::NoAction) => {},
//                                    Ok(JobResult::Complete {variables}) => {
//                                        println!("complete job");
//                                        let complete_job = CompleteJob { job_key, variables };
//                                        client.complete_job(complete_job).await.unwrap();
//                                    },
//                                    Ok(JobResult::Fail {error_message}) => {
//                                        client.fail_job(job_key, retries - 1).await.unwrap();
//                                    }
//                                    Err(_) => {
//                                        match panic_option {
//                                            PanicOption::DoNothingOnPanic => {
//                                            },
//                                            PanicOption::FailJobOnPanic => {
//                                            }
//                                        }
//                                    },
//                                };
//                            }
//                        });
//                        futures::future::join_all(it).await;
//                    }
                }
            });
            futures::future::join_all(i).await;
        }
    }
}

//
//impl<I, Fut1> WorkerBuilder<I, Fut1>
//    where
//        I: Stream + Unpin,
//        Fut1: Future<Output = JobResult> + Send + UnwindSafe + 'static,
//{
////    pub fn with_interval(interval: I) -> Self {
////        WorkerBuilder {
////            interval,
////            workers: HashMap::new(),
////        }
////    }
//
////    pub fn set_worker<F: Fn(ActivatedJob) -> Fut1 + Send + Sync + 'static>(mut self, wc: WorkerConfig, f: F) -> Self {
////        let job_type = wc.job_type.clone();
////        self.workers.insert(job_type, (wc, Arc::new(f)));
////        self
////    }
//
//    pub async fn build(self, client: Arc<Client>) -> () {
//        let mut interval = self.interval;
//        let job_handlers: Vec<(String, WorkerConfig, Arc<dyn Fn(ActivatedJob) -> Fut1 + Send + Sync>)> = self.workers.into_iter().map(|(jt,(wc, f))| (jt, wc, f)).collect();
//        while let Some(_) = interval.next().await {
//            let activate_jobs = ActivateJobs::new("the_worker", "the_job_type", 10000, 10);
//            for jh in job_handlers.iter() {
//                let (job_type, wc, f): &(String, WorkerConfig, Arc<dyn Fn(ActivatedJob) -> Fut1 + Send + Sync>) = jh;
////                let client = self.client.clone().unwrap();
//                runtime::spawn(async move {
//                    let activate_jobs = ActivateJobs::new(wc.worker_name.clone(), job_type.to_string(), wc.timeout, wc.max_jobs_to_activate);
//                    let panic_option = wc.panic_option;
//                    let mut activate_jobs_stream = client.activate_jobs(activate_jobs);
//                    while let Some(Ok(activated_jobs)) = activate_jobs_stream.next().await {
//                        for aj in activated_jobs.activated_jobs.into_iter() {
//                            let f = f.clone();
//                            let klient = client.clone();
//                            let job_key = aj.key;
//                            let retries = aj.retries;
//                            runtime::spawn(async move {
//                                match f(aj).catch_unwind().await {
//                                    Ok(JobResult::NoAction) => {},
//                                    Ok(JobResult::Complete {variables}) => {
//                                        let complete_job = CompleteJob { job_key, variables };
//                                        klient.complete_job(complete_job).await.unwrap();
//                                    },
//                                    Ok(JobResult::Fail {error_message}) => {
//                                        klient.fail_job(job_key, retries - 1).await.unwrap();
//                                    }
//                                    Err(_) => {
//                                        match panic_option {
//                                            PanicOption::DoNothingOnPanic => {
//                                            },
//                                            PanicOption::FailJobOnPanic => {
//                                            }
//                                        }
//                                    },
//                                };
//                            });
//                        };
//                    }
//                });
//            }
//        }
//        ()
//    }
//}