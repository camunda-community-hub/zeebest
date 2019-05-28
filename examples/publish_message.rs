#[macro_use]
extern crate serde_derive;

use futures::Future;
use zeebest::Client;

#[derive(Serialize)]
struct Payment {
    #[serde(rename = "total-charged")]
    pub total_charged: f32,
}

fn main() {
    let client = Client::new().unwrap();

    let payment = Payment {
        total_charged: 25.95,
    };

    let result = client
        .publish_message("payment-confirmed", "10", 10000, "messageId", payment)
        .wait()
        .unwrap();

    println!("publish message result: {:?}", result);
}
