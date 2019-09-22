#[macro_use]
extern crate serde_derive;

use atomic_counter::{AtomicCounter, RelaxedCounter};
use futures::compat::Stream01CompatExt;
use futures::prelude::*;

use std::sync::Arc;
use std::time::Duration;
use structopt::StructOpt;
use zeebest::{Client, JobResult, PanicOption, PublishMessage, WorkflowInstance, WorkflowVersion};

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
    PlaceOrder {
        #[structopt(short = "c", long = "count")]
        count: i32,
    },
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

fn main() {
    let client = Client::new("127.0.0.1", 26500).expect("Could not connect to broker.");

    let opt = Opt::from_args();

    match opt {
        Opt::DeployWorkflow => {
            futures::executor::block_on(client.deploy_bpmn_workflow(
                "order-process",
                include_bytes!("../examples/order-process.bpmn").to_vec(),
            ))
            .unwrap();
        }
        Opt::PlaceOrder { count } => {
            for _ in 0..count {
                futures::executor::block_on(client.create_workflow_instance(
                    WorkflowInstance::workflow_instance_with_bpmn_process(
                        "order-process",
                        WorkflowVersion::Latest,
                    ),
                ))
                .unwrap();
            }
        }
        Opt::NotifyPaymentReceived { order_id, cost } => {
            futures::executor::block_on(
                client.publish_message(
                    PublishMessage::new(
                        "payment-received",
                        order_id.to_string().as_str(),
                        10000,
                        "msgid",
                    )
                    .variables(&Payment { order_value: cost })
                    .unwrap(),
                ),
            )
            .unwrap();
        }
        Opt::ProcessJobs => {
            let order_id_counter = Arc::new(RelaxedCounter::new(0));

            let initial_payment_handler = move |_| {
                let order_id_counter = order_id_counter.clone();
                let order_id = order_id_counter.inc();
                let variables = serde_json::to_string(&Order {
                    order_id: order_id as i32,
                })
                .unwrap();
                let job_result = JobResult::Complete {
                    variables: Some(variables),
                };
                futures::future::ready(job_result).boxed()
            };

            let initiate_payment_job = zeebest::JobWorker::new(
                "rusty-worker".to_string(),
                "initiate-payment".to_string(),
                Duration::from_secs(3).as_secs() as _,
                1,
                PanicOption::FailJobOnPanic,
                client.clone(),
                initial_payment_handler,
            );

            let ship_without_insurance_job = zeebest::JobWorker::new(
                "rusty-worker".to_string(),
                "ship-without-insurance".to_string(),
                Duration::from_secs(3).as_secs() as _,
                1,
                PanicOption::FailJobOnPanic,
                client.clone(),
                |_| futures::future::ready(JobResult::Complete { variables: None }).boxed(),
            );

            let ship_with_insurance_job = zeebest::JobWorker::new(
                "rusty-worker".to_string(),
                "ship-with-insurance".to_string(),
                Duration::from_secs(3).as_secs() as _,
                1,
                PanicOption::FailJobOnPanic,
                client.clone(),
                |_| futures::future::ready(JobResult::Complete { variables: None }).boxed(),
            );

            let interval = tokio::timer::Interval::new_interval(Duration::from_secs(4));

            let f = interval.compat().for_each(|_| {
                let s1 = initiate_payment_job.clone().activate_and_process_jobs();
                let s2 = ship_with_insurance_job.clone().activate_and_process_jobs();
                let s3 = ship_without_insurance_job
                    .clone()
                    .activate_and_process_jobs();
                futures::future::join3(s1, s2, s3).then(|_| futures::future::ready(()))
            });

            futures::executor::block_on(f);
        }
    }
}
