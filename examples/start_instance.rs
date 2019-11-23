#[macro_use]
extern crate serde_derive;

use zeebest::{Client, WorkflowInstance, WorkflowVersion};

#[derive(Serialize)]
struct PlaceOrder {
    #[serde(rename = "orderId")]
    pub order_id: i32,
}

#[runtime::main]
async fn main() {
    let client = Client::new("127.0.0.1:26500").unwrap();
    let place_order = PlaceOrder { order_id: 10 };
    let workflow_instance = WorkflowInstance::workflow_instance_with_bpmn_process(
        "simple-process",
        WorkflowVersion::Latest,
    )
    .variables(&place_order)
    .unwrap();
    let result = client.create_workflow_instance(workflow_instance).await;
    println!("create workflow result: {:?}", result);
}
