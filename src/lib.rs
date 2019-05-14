#[macro_use] extern crate failure;

pub mod gateway;
pub mod gateway_grpc;

use crate::gateway_grpc::*;
//use crate::gateway::*;

//use grpc::ClientStub;
use grpc::ClientStubExt;
use crate::gateway::{TopologyResponse, ListWorkflowsResponse, WorkflowMetadata};

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Gateway Error")]
    GatewayError(grpc::Error),
    #[fail(display = "Topology Error")]
    TopologyError(grpc::Error),
    #[fail(display = "List Workflows Error")]
    ListWorkflowsError(grpc::Error),
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

    /// Get the topology. The returned struct is similar to what is printed when running `zbctl status`.
    pub fn topology(&self) -> Result<TopologyResponse, Error> {
        let options = Default::default();
        let topology_request = Default::default();
        let grpc_response: grpc::SingleResponse<_> = self.gateway_client.topology(options, topology_request);
        let topology_response = grpc_response.wait_drop_metadata().map_err(|e| Error::TopologyError(e))?;
        Ok(topology_response)
    }

    // list the workflows
    pub fn list_workflows(&self) -> Result<Vec<WorkflowMetadata>, Error> {
        let options = Default::default();
        let list_workflows_request = Default::default();
        let grpc_response: grpc::SingleResponse<_> = self.gateway_client.list_workflows(options, list_workflows_request);
        let list_workflows_response: ListWorkflowsResponse = grpc_response.wait_drop_metadata().map_err(|e| Error::ListWorkflowsError(e))?;
        let workflows: Vec<WorkflowMetadata> = list_workflows_response.workflows.into();
        Ok(workflows)
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

    #[test]
    fn check_list_workflows() {
        let client = Client::new().unwrap();
        let workflows = client.list_workflows().unwrap();
        println!("{:?}", workflows);
    }
}
