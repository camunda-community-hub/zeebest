#[macro_use]
extern crate serde_derive;

use atomic_counter::{AtomicCounter, RelaxedCounter};
use futures::stream::Stream;
use futures::Future;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use structopt::StructOpt;
use tokio::timer::Interval;
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

fn main() {
    let mut client = Client::new("127.0.0.1", 26500).expect("Could not connect to broker.");

    let opt = Opt::from_args();
    match opt {
        Opt::DeployWorkflow => {
            client
                .deploy_bpmn_workflow(
                    "order-process",
                    include_bytes!("../examples/order-process.bpmn").to_vec(),
                )
                .wait()
                .unwrap();
        }
        Opt::PlaceOrder => {
            client
                .create_workflow_instance(WorkflowInstance::workflow_instance_with_bpmn_process(
                    "order-process",
                    WorkflowVersion::Latest,
                ))
                .wait()
                .unwrap();
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
                )
                .wait()
                .unwrap();
        }
        Opt::ProcessJobs => {
            let order_id_counter = Arc::new(RelaxedCounter::new(0));

            // a payment worker that passes along a new order_id
            let mut initiate_payment_worker = client.worker(
                "rusty-worker",
                "initiate-payment",
                3000,
                4,
                PanicOption::FailJobOnPanic,
                move |_| {
                    sleep(Duration::from_secs(5));
                    Ok({
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
                    })
                },
            );

            let mut ship_without_insurance_worker = client.worker(
                "rusty-worker",
                "ship-without-insurance",
                3000,
                4,
                PanicOption::FailJobOnPanic,
                |_| Ok(JobResult::Complete { variables: None }),
            );

            let mut ship_with_insurance_worker = client.worker(
                "rusty-worker",
                "ship-with-insurance",
                3000,
                4,
                PanicOption::FailJobOnPanic,
                |_| Ok(JobResult::Complete { variables: None }),
            );

            let f = Interval::new_interval(Duration::from_secs(1))
                .map_err(|_| panic!("The interval panicked."))
                .and_then(move |_| {
                    let s1 = initiate_payment_worker.activate_and_process_jobs();
                    let s2 = ship_without_insurance_worker.activate_and_process_jobs();
                    let s3 = ship_with_insurance_worker.activate_and_process_jobs();
                    let sa = s1.select(s2);
                    let sb = sa.select(s3);
                    sb.map(|jr| {
                        println!("job result: {:?}", jr);
                    })
                    .map_err(|e| {
                        eprintln!("job errored: {:?}", e);
                    })
                    .collect()
                    .map(|_| ())
                })
                .collect()
                .map(|_| println!("Done."))
                .map_err(|_| ());

            tokio::run(f);
        }
    }
}
