#[macro_use]
extern crate serde_derive;

use futures::executor::block_on;
use zeebest::{Client, WorkflowInstance, WorkflowVersion};

#[derive(Serialize)]
struct PlaceOrder {
    #[serde(rename = "orderId")]
    pub order_id: i32,
}

fn main() {
    let client = Client::new("127.0.0.1", 26500).unwrap();

    let place_order = PlaceOrder { order_id: 10 };

    let workflow_instance = WorkflowInstance::workflow_instance_with_bpmn_process(
        "simple-process",
        WorkflowVersion::Latest,
    )
    .variables(&place_order)
    .unwrap();

    let result = block_on(client.create_workflow_instance(workflow_instance));

    println!("create workflow result: {:?}", result);
}
