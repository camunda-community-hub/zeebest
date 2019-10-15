#[macro_use]
extern crate serde_derive;

use tonic::transport::Certificate;
use std::path::PathBuf;
use std::fs;
use tonic::transport::{ Channel, ClientTlsConfig };
use tonic::codegen::http::Uri;
use zeebest::gateway::client::GatewayClient;

#[derive(Deserialize, Debug)]
struct Config {
    server_url: String,
    certificate_path: PathBuf,
    client_id: String,
    client_secret: String,
}

#[derive(Serialize, Debug)]
struct AuthzRequest {
    client_id: String,
    client_secret: String,
    audience: String,
    grant_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AuthzResponse {
    access_token: String,
    expires_in: u64,
    token_type: String,
    scope: String,
}

#[tokio::main]
async fn main() {
    // parse configuration from environment variables
    let Config { server_url, certificate_path, client_id, client_secret } = envy::from_env().unwrap();

    let domain_name = server_url.split(":").next().unwrap().to_owned();
    let certificate_data = fs::read(certificate_path).unwrap();
    let certificate = Certificate::from_pem(certificate_data);

    // get me a token
    let client = reqwest::Client::new();
    let request_json = AuthzRequest {
        client_id: client_id.clone(),
        client_secret: client_secret.clone(),
        audience: domain_name.clone(),
        grant_type: "client_credentials".to_string()
    };

    let response: AuthzResponse = client.post("https://login.cloud.camunda.io/oauth/token/")
        .json(&request_json)
        .send()
        .await.unwrap()
        .json().await.unwrap();

    let tls_config = ClientTlsConfig::with_rustls()
        .ca_certificate(certificate)
        .domain_name(domain_name)
        .clone();

    let uri = format!("https://{}", server_url).parse::<Uri>().unwrap();
    println!("uri: {:?}", uri);

    let channel = Channel::builder(uri)
        .tls_config(&tls_config)
        .intercept_headers(move |headers| {
            let value = format!("Bearer {}", response.access_token).parse().unwrap();
            headers.insert("authorization",value);
        })
        .channel();

    let mut gateway_client = GatewayClient::new(channel);

    let request = tonic::Request::new(zeebest::gateway::TopologyRequest {});
    let result = gateway_client.topology(request).await.unwrap();
    println!("result: {:?}", result);
}
