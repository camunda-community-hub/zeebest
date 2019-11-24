#[macro_use]
extern crate serde_derive;

use atomic_counter::{AtomicCounter};

use futures::prelude::*;

use std::time::Duration;
use structopt::StructOpt;

use zeebest::gateway::{DeployWorkflowRequest, WorkflowRequestObject, CreateWorkflowInstanceRequest, PublishMessageRequest, ActivateJobsRequest};
use zeebest::gateway::workflow_request_object::ResourceType;

use tower::Service;
use futures::task::Context;
use futures::{Poll, FutureExt};
use std::pin::Pin;

type ActivatedJobs = Vec<zeebest::gateway::ActivatedJob>;

trait EventHandler {
    fn activated_jobs(&mut self, activated_jobs: ActivatedJobs) -> ActivatedJobs;
    fn request_jobs(&mut self) -> Option<ActivateJobsRequest>;
}

struct MyError;

struct JustOne;

impl Service<ActivatedJobs> for JustOne {
    type Response = ActivatedJobs;
    type Error = MyError;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        unimplemented!()
    }

    fn call(&mut self, req: Self::Response) -> Self::Future {
        futures::future::ok(req).boxed()
    }
}

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
        Opt::DeployWorkflow => {
            let deploy_workflow_request = DeployWorkflowRequest {
                workflows: vec![WorkflowRequestObject {
                    name: "order-process".to_string(),
                    definition: include_bytes!("../examples/order-process.bpmn").to_vec(),
                    r#type: ResourceType::Bpmn as i32,
                }]
            };

            client
                .deploy_workflow(deploy_workflow_request)
                .await
                .unwrap();
        }
        Opt::PlaceOrder { count } => {
            let create_workflow_instance_request= CreateWorkflowInstanceRequest {
                workflow_key: 0,
                bpmn_process_id: "order-process".to_string(),
                version: -1,
                variables: "".to_string()
            };

            for _ in 0..count {
                let create_workflow_instance_request= create_workflow_instance_request.clone();
                client
                    .create_workflow_instance(create_workflow_instance_request)
                    .await
                    .unwrap();
            }
        }
        Opt::NotifyPaymentReceived { order_id, cost } => {

            let public_message_request = PublishMessageRequest {
                name: "payment-received".to_string(),
                correlation_key: order_id.to_string(),
                time_to_live: 10000,
                message_id: "messageId".to_string(),
                variables: serde_json::to_string(&Payment { order_value: cost }).unwrap(),
            };

            client
                .publish_message(public_message_request)
                .await
                .unwrap();
        }
        Opt::ProcessJobs => {
            use async_std::sync::{Arc, RwLock};
            let safe_client = Arc::new(RwLock::new(client));
            let mut interval = runtime::time::Interval::new(Duration::from_secs(4));
            while let Some(_) = interval.next().await {
                let mut client = safe_client.write().await;
//                let client= &mut *client;

                let activate_job_request: ActivateJobsRequest = ActivateJobsRequest {
                    fetch_variable: vec![],
                    max_jobs_to_activate: 10,
                    request_timeout: 10000,
                        timeout: 10000,
                    r#type: "".to_string(),
                    worker: "".to_string(),
                };

                let response = client.activate_jobs(activate_job_request.clone()).await.unwrap();
                let mut activate_jobs_response = response.into_inner();

                while let Some(activate_jobs_result) = activate_jobs_response.next().await  {
                    match activate_jobs_result {
                        Ok(activate_jobs) => {
                            let _jobs = activate_jobs.jobs;

                        },
                        Err(_e) => {
//                            let _e = Error::TonicError(e);
                        }
                    }
                };




//                let f1 = initiate_payment_job.clone().activate_and_process_jobs();
//                let f2 = ship_with_insurance_job.clone().activate_and_process_jobs();
//                let f3 = ship_without_insurance_job
//                    .clone()
//                    .activate_and_process_jobs();
//                futures::future::join3(f1, f2, f3).await;
            }

//            let order_id_counter = Arc::new(RelaxedCounter::new(0));

//            let initial_payment_handler = move |_| {
//                let order_id_counter = order_id_counter.clone();
//                let order_id = order_id_counter.inc();
//                let variables = serde_json::to_string(&Order {
//                    order_id: order_id as i32,
//                })
//                .unwrap();
//                let job_result = JobResult::Complete {
//                    variables: Some(variables),
//                };
//                futures::future::ready(job_result).boxed()
//            };

//            let initiate_payment_job = zeebest::JobWorker::new(
//                "rusty-worker".to_string(),
//                "initiate-payment".to_string(),
//                Duration::from_secs(3).as_secs() as _,
//                1,
//                PanicOption::FailJobOnPanic,
//                client.clone(),
//                initial_payment_handler,
//            );
//
//            let ship_without_insurance_job = zeebest::JobWorker::new(
//                "rusty-worker".to_string(),
//                "ship-without-insurance".to_string(),
//                Duration::from_secs(3).as_secs() as _,
//                1,
//                PanicOption::FailJobOnPanic,
//                client.clone(),
//                |_| futures::future::ready(JobResult::Complete { variables: None }).boxed(),
//            );
//
//            let ship_with_insurance_job = zeebest::JobWorker::new(
//                "rusty-worker".to_string(),
//                "ship-with-insurance".to_string(),
//                Duration::from_secs(3).as_secs() as _,
//                1,
//                PanicOption::FailJobOnPanic,
//                client.clone(),
//                |_| futures::future::ready(JobResult::Complete { variables: None }).boxed(),
//            );

            let mut interval = runtime::time::Interval::new(Duration::from_secs(4));
            while let Some(_) = interval.next().await {
//                let f1 = initiate_payment_job.clone().activate_and_process_jobs();
//                let f2 = ship_with_insurance_job.clone().activate_and_process_jobs();
//                let f3 = ship_without_insurance_job
//                    .clone()
//                    .activate_and_process_jobs();
//                futures::future::join3(f1, f2, f3).await;
            }
        }
    }
}
