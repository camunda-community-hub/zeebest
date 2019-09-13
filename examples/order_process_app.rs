#[macro_use]
extern crate serde_derive;

use atomic_counter::{AtomicCounter, RelaxedCounter};
use futures::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use structopt::StructOpt;
use zeebest::{Client, PublishMessage, WorkflowInstance, WorkflowVersion, PanicOption, JobResult, WorkerConfig};
use zeebest::worker_builder::WorkerBuilder;
use runtime::time::Interval;
use failure::_core::mem::zeroed;

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

#[runtime::main]
async fn main() {
    let client = Client::new("127.0.0.1", 26500).expect("Could not connect to broker.");

    let opt = Opt::from_args();

    match opt {
        Opt::DeployWorkflow => {
            client
                .deploy_bpmn_workflow(
                    "order-process",
                    include_bytes!("../examples/order-process.bpmn").to_vec(),
                ).await.unwrap();
        }
        Opt::PlaceOrder { count } => {
            for _ in 0..count {
                client
                    .create_workflow_instance(WorkflowInstance::workflow_instance_with_bpmn_process(
                        "order-process",
                        WorkflowVersion::Latest,
                    )).await.unwrap();
            }
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

            let initial_payment_config = WorkerConfig::new(
                "rusty-worker".to_string(),
                "initiate-payment".to_string(),
                Duration::from_secs(3).as_secs() as _,
                1,
                PanicOption::FailJobOnPanic,
            );

            let initial_payment_handler = move |_| {
                let order_id_counter = order_id_counter.clone();
                async move {
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

            let ship_without_insurance_config = WorkerConfig::new(
                "rusty-worker".to_string(),
                "ship-without-insurance".to_string(),
                Duration::from_secs(3).as_secs() as _,
                1,
                PanicOption::FailJobOnPanic,
            );

            let ship_with_insurance_config = WorkerConfig::new(
                "rusty-worker".to_string(),
                "ship-with-insurance".to_string(),
                Duration::from_secs(3).as_secs() as _,
                1,
                PanicOption::FailJobOnPanic,
            );

            let initial_payment_job = zeebest::worker_builder::Job::new(initial_payment_handler,
                                                        client.clone(),
                                                        initial_payment_config);

            let ship_with_insurance_job = zeebest::worker_builder::Job::new(|_| futures::future::ready(JobResult::Complete { variables: None }).boxed(),
                                                                                     client.clone(),
                                                   ship_with_insurance_config);

            let mut interval = Interval::new(Duration::from_secs(4));
            while let Some(_) = interval.next().await {
                let s1 = initial_payment_job.clone().activate_and_process_jobs();
                let s2 = ship_with_insurance_job.clone().activate_and_process_jobs();
                futures::future::join(s1, s2).await;
            }


//            WorkerBuilder::new_with_interval_and_client(Interval::new(Duration::from_secs(5)), client)
//                .add_job_handler("initiate-payment", initial_payment_config, initial_payment_handler)
//                .add_job_handler("ship-without-insurance", ship_without_insurance_config, |_| async { JobResult::Complete { variables: None } }.boxed())
//                .add_job_handler("ship-with-insurance", ship_with_insurance_config, |_| async { JobResult::Complete { variables: None } }.boxed())
//                .into_future().await;
        }
    }
}
