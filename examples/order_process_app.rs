#[macro_use]
extern crate serde_derive;

use atomic_counter::{AtomicCounter, RelaxedCounter};
use futures::{Future, TryFuture, TryFutureExt, Stream, TryStream, TryStreamExt, StreamExt};
use futures::prelude::*;
use runtime::net::TcpListener;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use structopt::StructOpt;
use zeebest::{Client, PublishMessage, WorkflowInstance, WorkflowVersion, ActivateJobs, ActivatedJob, PanicOption, ActivatedJobs, JobResult, CompleteJob, WorkerConfig};
use std::pin::Pin;
use std::io;
use futures::compat::Stream01CompatExt;
use std::panic::UnwindSafe;
use zeebest::Error::FailJobError;
use zeebest::worker_builder::WorkerBuilder;
use futures::executor::block_on;
use runtime::time::Interval;

#[derive(StructOpt, Debug)]
#[structopt(
    about = "An app for processing orders. This can deploy the workflow, place orders, notify of payment, or be a job worker."
)]
enum Opt {
    #[structopt(
        name = "deploy",
        about = "Deploy the workflow on the broker. You probably only need to do this once."
    )]
    DeployWorkflow,
    #[structopt(
        name = "place-order",
        about = "Place a new order. This starts a workflow instance."
    )]
    PlaceOrder,
    #[structopt(
        name = "notify-payment-received",
        about = "Indicate that the order was processed and there is now a cost for the order."
    )]
    NotifyPaymentReceived {
        #[structopt(short = "i", long = "order-id")]
        order_id: i32,
        #[structopt(short = "c", long = "cost")]
        cost: f32,
    },
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

//async fn sum_with_try_next(
//    mut stream: Pin<&mut dyn Stream<Item = Result<i32, io::Error>>>,
//) -> Result<i32, io::Error> {
//    use futures::stream::TryStreamExt; // for `try_next`
//    let mut sum = 0;
//    while let Some(item) = stream.try_next().await? {
//        sum += item;
//    }
//    Ok(sum)
//}

//async fn interval<Fut1,F>(client: Arc<Client>, f: Arc<F>, panic_option: PanicOption, mut interval_stream: impl Stream + Unpin) -> Result<i32, ()> where
//    Fut1: Future<Output = JobResult> + Send + UnwindSafe + 'static,
//    F: Fn(ActivatedJob) -> Fut1 + Send + Sync + 'static,
//{
////    let mut stream = Interval::new_interval(Duration::from_secs(1)).compat();
//    while let Some(_) = interval_stream.next().await {
//        let activate_jobs = ActivateJobs::new("the_worker", "the_job_type", 10000, 10);
//        let mut activate_jobs_stream= client.activate_jobs(activate_jobs);
//        while let Some(Ok(activated_jobs)) = activate_jobs_stream.next().await {
//            for aj in activated_jobs.activated_jobs.into_iter() {
//                let f = f.clone();
//                let klient = client.clone();
//                let job_key = aj.key;
//                let retries = aj.retries;
//                runtime::spawn(async move {
//                    let r = match f(aj).catch_unwind().await {
//                        Ok(r) => r,
//                        Err(_) => {
//                            match panic_option {
//                                PanicOption::DoNothingOnPanic => {
//                                    JobResult::Fail {
//                                        error_message: Some("Job Handler Panicked.".to_string()),
//                                    }
//                                },
//                                PanicOption::FailJobOnPanic => {
//                                    JobResult::NoAction
//                                }
//                            }
//                        },
//                    };
//                    match r {
//                        JobResult::NoAction => {},
//                        JobResult::Complete {variables} => {
//                            let complete_job = CompleteJob { job_key, variables };
//                            klient.complete_job(complete_job).await.unwrap();
//                        },
//                        JobResult::Fail {error_message} => {
//                            klient.fail_job(job_key, retries - 1).await.unwrap();
//                        }
//                    }
//                });
//            };
//        }
//    }
//    Ok(0)
//}

//            .and_then(move |(result, activated_job)| {
//                let result: JobResult = result;
//                let cloned_result_complete = result.clone();
//                let cloned_result_fail = result.clone();
//                let job_key = activated_job.key;
//                let retries = activated_job.retries;
//                match result {
//                    JobResult::NoAction => {
//                        JobResultFuture::NoAction(futures::future::ok((result, activated_job)))
//                    }
//                    JobResult::Fail { error_message } => {
//                        let options = Default::default();
//                        let mut fail_request = gateway::FailJobRequest::default();
//                        fail_request.set_jobKey(job_key);
//                        fail_request.set_retries(retries - 1);
//                        if let Some(error_message) = error_message {
//                            fail_request.set_errorMessage(error_message);
//                        }
//                        JobResultFuture::Fail(
//                            gateway_client
//                                .fail_job(options, fail_request)
//                                .drop_metadata()
//                                .map_err(|e| Error::FailJobError(e))
//                                .map(move |_| (cloned_result_complete, activated_job)),
//                        )
//                    }
//                    JobResult::Complete { variables } => {
//                        let options = Default::default();
//                        let mut complete_request = gateway::CompleteJobRequest::default();
//                        complete_request.set_jobKey(job_key);
//                        if let Some(variables) = variables {
//                            complete_request.set_variables(variables)
//                        }
//                        JobResultFuture::Complete(
//                            gateway_client
//                                .complete_job(options, complete_request)
//                                .drop_metadata()
//                                .map_err(|e| Error::CompleteJobError(e))
//                                .map(move |_| (cloned_result_fail, activated_job)),
//                        )
//                    }
//                }

#[runtime::main]
async fn main() {
    let mut client = Client::new("127.0.0.1", 26500).expect("Could not connect to broker.");

//    let opt = Opt::from_args();
    let opt = Opt::ProcessJobs;

    match opt {
        Opt::DeployWorkflow => {
            client
                .deploy_bpmn_workflow(
                    "order-process",
                    include_bytes!("../examples/order-process.bpmn").to_vec(),
                ).await.unwrap();
        }
        Opt::PlaceOrder => {
            client
                .create_workflow_instance(WorkflowInstance::workflow_instance_with_bpmn_process(
                    "order-process",
                    WorkflowVersion::Latest,
                )).await.unwrap();
        }
        Opt::NotifyPaymentReceived { order_id, cost } => {
            client
                .publish_message(
                    PublishMessage::new(
                        "payment-received",
                        order_id.to_string().as_str(),
                        10000,
                        "msgid",
                    )
                    .variables(&Payment { order_value: cost })
                    .unwrap(),
                ).await.unwrap();
        }
        Opt::ProcessJobs => {
            let order_id_counter = Arc::new(RelaxedCounter::new(0));

            let f = |aj: ActivatedJob| { async { JobResult::NoAction } };
            let f = Arc::new(f);

            let initial_payment_config = WorkerConfig::new(
                "rusty-worker".to_string(),
                "initiate-payment".to_string(),
                Duration::from_secs(3).as_millis() as _,
                4,
                PanicOption::FailJobOnPanic,
            );

            let initial_payment_handler = move |_| {
                let order_id_counter = order_id_counter.clone();
                async move {
                    println!("doing work");
                    // increment the order id counter
                    // this would normally be a key in a database or something
                    let order_id = order_id_counter.inc();
                    let variables = serde_json::to_string(&Order {
                        order_id: order_id as i32,
                    })
                        .unwrap();
                    JobResult::Complete {
                        variables: Some(variables),
                    }
                }.boxed()
            };


            println!("processing jobs");
            WorkerBuilder::new_with_interval_and_client(Interval::new(Duration::from_secs(5)), Arc::new(client))
                .add_job_handler("initiate-payment", initial_payment_config, initial_payment_handler)
                .into_future().await;
        }
    }
}
