#[macro_use] extern crate failure;

pub mod gateway;
pub mod gateway_grpc;

use crate::gateway_grpc::*;
//use crate::gateway::*;

//use grpc::ClientStub;
use grpc::ClientStubExt;
use crate::gateway::{TopologyResponse};

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Gateway Error")]
    GatewayError(grpc::Error),
    #[fail(display = "Topology Error")]
    TopologyError(grpc::Error),
}

pub struct Client {
    gateway_client: GatewayClient,
}

impl Client {
    pub fn new() -> Result<Self, Error> {
        let config = Default::default();
        let gateway_client = GatewayClient::new_plain("127.0.0.1", 26500, config)
            .map_err(|e| Error::GatewayError(e))?;
        Ok(Self {
            gateway_client,
        })
    }

    pub fn topology(&self) -> Result<TopologyResponse, Error> {
        let options = Default::default();
        let topology_request = Default::default();
        let grpc_response: grpc::SingleResponse<_> = self.gateway_client.topology(options, topology_request);
        let topology_response = grpc_response.wait_drop_metadata().map_err(|e| Error::TopologyError(e))?;
        Ok(topology_response)
    }
}

#[cfg(test)]
mod tests {
    use crate::Client;

    #[test]
    fn check_topology() {
        let client = Client::new().unwrap();
        let topology = client.topology().unwrap();
        println!("{:?}", topology);
    }
}
