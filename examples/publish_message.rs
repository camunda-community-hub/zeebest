#[macro_use]
extern crate serde_derive;

use zeebest::gateway::PublishMessageRequest;

#[derive(Serialize)]
struct Payment {
    #[serde(rename = "total-charged")]
    pub total_charged: f32,
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

    let payment = Payment {
        total_charged: 25.95,
    };

    let request = PublishMessageRequest {
        name: "payment-confirmed".to_string(),
        correlation_key: "10".to_string(),
        time_to_live: 10000,
        message_id: "messageId".to_string(),
        variables: serde_json::to_string(&payment).unwrap(),
    };

    client.publish_message(request).await.unwrap();
    println!("published message");
}
