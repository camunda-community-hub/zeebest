pub use crate::gateway;


use std::sync::Arc;

use crate::client::client_builder::ClientBuilder;
use async_std::sync::RwLock;

/// The primary type for interacting with zeebe.
#[derive(Clone)]
pub struct Client {
    pub(crate) internal_client:
        Arc<RwLock<gateway::client::GatewayClient<tonic::transport::Channel>>>,
}

impl Client {
    /// Construct a new `Client` that connects to a broker with `host` and `port`.
    pub fn builder(uri: http::Uri) -> ClientBuilder {
        let mut builder: ClientBuilder = Default::default();
        builder.uri = Some(uri);
        builder
    }
}
