#[macro_use]
extern crate serde_derive;


use structopt::StructOpt;

use zeebest::gateway::{DeployWorkflowRequest, WorkflowRequestObject, CreateWorkflowInstanceRequest, PublishMessageRequest};
use zeebest::gateway::workflow_request_object::ResourceType;
use bastion::prelude::*;
use async_std::sync::Arc;
use serde::Serialize;

//use std::collections::HashSet;

//enum Message {
//    NewJobsActivated(Vec<zeebest::gateway::ActivatedJob>),
//    StartRunningJob(zeebest::gateway::ActivatedJob),
//    RunningJobReportsCompleted(zeebest::gateway::CompleteJobRequest),
//    RunningJobReportsFailed(zeebest::gateway::FailJobRequest),
//    RunningJobTimeout(i64),
//    ActivatedJobGetsRejected(zeebest::gateway::FailJobRequest),
//}

//struct State {
//    active_job_ids: HashSet<i64>,
//    max_job_count: usize,
//}

//impl State {
    /// Add jobs to the active set
//    pub fn _new_activated_jobs(&mut self, _all_new_activated_jobs: Vec<zeebest::gateway::ActivatedJob>) {
//        let
//
//        let all_new_activated_job_ids: HashSet<i64> = all_new_activated_jobs.iter().map(|aj| aj.key.clone()).collect();
//         if all_new_activated_job_ids.len() >= self.max_job_count {}
//         else {
//             let take_count = self.max_job_count - all_new_activated_job_ids.len();
//             let accepted_activated_job_ids: HashSet<i64> = all_new_activated_job_ids.into_iter().take(take_count).collect();
//             self.active_job_ids = self.active_job_ids.union(&accepted_activated_job_ids).cloned().collect();
//             let accepted_activated_jobs: Vec<zeebest::gateway::ActivatedJob> = all_new_activated_jobs.into_iter().filter(|aj| accepted_activated_job_ids.contains(&aj.key)).collect();
//         }
//    }
//}

#[derive(StructOpt, Debug)]
#[structopt(
    about = "An app for processing orders. This can deploy the workflow, place orders, notify of payment, or be a job worker."
)]
enum Opt {
    #[structopt(
        name = "process-jobs",
        about = "Process all of the jobs on an interval. Will run forever. Print job results."
    )]
    ProcessJobs,
}

#[derive(Serialize)]
struct Order {
    #[serde(rename = "orderId")]
    pub order_id: i32,
}

#[derive(Serialize)]
struct Payment {
    #[serde(rename = "orderValue")]
    pub order_value: f32,
}

#[tokio::main]
async fn main() {
    let uri: http::Uri = "http://127.0.0.1:26500"
        .parse::<http::Uri>()
        .unwrap();
    let mut client = zeebest::Client::builder(uri)
        .connect()
        .await
        .unwrap();

    let opt = Opt::from_args();

    match opt {
        Opt::ProcessJobs => {
            Bastion::init();
            do_bastion(1);
            Bastion::start();
            Bastion::block_until_stopped();
        }
    }
}

struct MyError {
    message: Option<String>
}

fn create_bastion_workers<H, R>(redundancy: usize, handler: Arc<H>) -> Result<ChildrenRef, ()>
where
    H: Fn(zeebest::gateway::ActivatedJob) -> Result<R, MyError> + Send + Sync + 'static,
    R: Serialize,
{
    Bastion::children(|children: Children| {
        let callbacks = Callbacks::new()
            .with_before_start(move || {
                println!("before_start");
            })
            .with_before_restart(move || {
                println!("before_restart");
                Bastion::stop(); // only ever run once.
            });

        children
            .with_redundancy(redundancy) // number of workers
            .with_callbacks(callbacks)
            .with_exec(move |ctx: BastionContext| {
                let handler = handler.clone();
                async move {
                    println!("Worker started!");
                    // Start receiving work
                    loop {
                        msg! { ctx.recv().await?,
                            msg: zeebest::gateway::ActivatedJob =!> {
                                panic!("oh no there was a problem in the happy case");
                                match handler(msg) {
                                    Ok(_json) => {
                                        let _ = answer!("json".to_string());
                                    },
                                    Err(_) => {
                                        let _ = answer!("failure".to_string());
                                    },
                                };
                            };
                            _: _ => ();
                        }
                    }
                }
            })
    })
}

fn do_bastion(redundancy: usize) {
    // Workers that process the work.
    let do_stuff = Arc::new(|_aj| {
        Ok("hooray".to_string())
    });
    let workers = create_bastion_workers(redundancy, do_stuff).expect("Couldn't start a new children group.");

    // Get a shadowed sharable reference of workers.
    let workers = Arc::new(workers);

    // Mapper that generates work.
    Bastion::children(|children: Children| {
        children.with_exec(move |_ctx: BastionContext| {
            let workers = workers.clone();
            async move {
                println!("Mapper started!");
                // Distribute your workload to workers
                for id_worker_pair in workers.elems().iter().enumerate() {
                    let job = zeebest::gateway::ActivatedJob::default();
                    let computed: Answer = id_worker_pair.1.ask(job).unwrap();
                    msg! { computed.await?,
                        msg: String => {
                            // Handle the answer..
                            println!("got response: {:?}", msg);
                        };
                        _: _ => ();
                    }
                }

                // Send a signal to system that computation is finished.
                Bastion::stop();

                Ok(())
            }
        })
    })
        .expect("Couldn't start a new children group.");
}


