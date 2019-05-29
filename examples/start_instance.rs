#[macro_use]
extern crate serde_derive;

use futures::Future;
use zeebest::{Client, WorkflowVersion};

#[derive(Serialize)]
struct PlaceOrder {
    #[serde(rename = "orderId")]
    pub order_id: i32,
}

fn main() {
    let client = Client::new("127.0.0.1", 26500).unwrap();

    let place_order = PlaceOrder { order_id: 10 };

    let result = client
        .create_workflow_instance("simple-process", WorkflowVersion::Latest, place_order)
        .wait()
        .unwrap();

    println!("create workflow result: {:?}", result);
}
