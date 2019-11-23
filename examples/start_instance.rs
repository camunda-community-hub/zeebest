#[macro_use]
extern crate serde_derive;

use zeebest::{WorkflowInstance, WorkflowVersion};

#[derive(Serialize)]
struct PlaceOrder {
    #[serde(rename = "orderId")]
    pub order_id: i32,
}

#[tokio::main]
async fn main() {
    let uri: http::Uri = "http://127.0.0.1:26500"
        .parse::<http::Uri>()
        .unwrap();
    let client: zeebest::Client = zeebest::Client::builder()
        .uri(uri)
        .connect()
        .await
        .unwrap();

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
