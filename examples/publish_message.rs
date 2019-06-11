#[macro_use]
extern crate serde_derive;

use futures::Future;
use zeebest::{Client, PublishMessage};

#[derive(Serialize)]
struct Payment {
    #[serde(rename = "total-charged")]
    pub total_charged: f32,
}

fn main() {
    let client = Client::new("127.0.0.1", 26500).unwrap();

    let payment = Payment {
        total_charged: 25.95,
    };

    let publish_message = PublishMessage::new("payment-confirmed", "10", 10000, "messageId")
        .variables(&payment)
        .unwrap();

    client
        .publish_message(publish_message)
        .wait()
        .unwrap();

    println!("published message");
}
