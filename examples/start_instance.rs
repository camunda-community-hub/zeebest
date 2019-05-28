use futures::Future;
use zeebe_client::client::Client;

fn main() {
    let client = Client::new().unwrap();
    let result = client
        .create_workflow_instance("simple-process".to_string(), "".to_string())
        .wait()
        .unwrap();
    println!("create workflow result: {:?}", result);
}
