#[macro_use]
extern crate serde_derive;
use zeebest::{PublishMessage};

#[derive(Serialize)]
struct Payment {
    #[serde(rename = "total-charged")]
    pub total_charged: f32,
}

#[runtime::main]
async fn main() {
    let uri: http::Uri = "http://127.0.0.1:26500"
        .parse::<http::Uri>()
        .unwrap();
    let client: zeebest::Client = zeebest::Client::builder()
        .uri(uri)
        .connect()
        .await
        .unwrap();

    let payment = Payment {
        total_charged: 25.95,
    };

    let publish_message = PublishMessage::new("payment-confirmed", "10", 10000, "messageId")
        .variables(&payment)
        .unwrap();

    client.publish_message(publish_message).await.unwrap();
    println!("published message");
}
