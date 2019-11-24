#[macro_use]
extern crate serde_derive;

use zeebest::gateway::{CreateWorkflowInstanceRequest};

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
    let mut client = zeebest::Client::builder(uri)
        .connect()
        .await
        .unwrap();

    let place_order = PlaceOrder { order_id: 10 };

    let create_workflow_instance_request = CreateWorkflowInstanceRequest {
        workflow_key: 0,
        bpmn_process_id: "simple-process".to_string(),
        version: -1,
        variables: serde_json::to_string(&place_order).unwrap()
    };

    let result = client.create_workflow_instance(create_workflow_instance_request).await.unwrap();
    println!("create workflow result: {:?}", result);
}
