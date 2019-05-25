use crate::gateway_grpc::*;
//use crate::gateway::*;

//use grpc::ClientStub;
use crate::activate_jobs::{ActivateJobs, ActivateJobsConfig};
use crate::complete_job::{CompleteJob, CompletedJobData};
pub use crate::gateway::{
    ActivateJobsResponse, ActivatedJob, CreateWorkflowInstanceRequest, WorkflowRequestObject,
};
use crate::gateway::{
    CreateWorkflowInstanceResponse, DeployWorkflowRequest, DeployWorkflowResponse,
    ListWorkflowsResponse, PublishMessageRequest, TopologyResponse, WorkflowMetadata,
};
use crate::worker::{Worker, WorkerConfig};
use grpc::ClientStubExt;
use std::sync::Arc;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Gateway Error. {:?}", _0)]
    GatewayError(grpc::Error),
    #[fail(display = "Topology Error")]
    TopologyError(grpc::Error),
    #[fail(display = "List Workflows Error")]
    ListWorkflowsError(grpc::Error),
    #[fail(display = "Deploy Workflow Error")]
    DeployWorkflowError(grpc::Error),
    #[fail(display = "Create Workflow Instance Error")]
    CreateWorkflowInstanceError(grpc::Error),
    #[fail(display = "Activate Job Error")]
    ActivateJobError(grpc::Error),
    #[fail(display = "Complete Job Error")]
    CompleteJobError(grpc::Error),
    #[fail(display = "Publish Message Error")]
    PublishMessageError(grpc::Error),
}

#[derive(Clone)]
pub struct Client {
    pub(crate) gateway_client: Arc<GatewayClient>,
}

impl Client {
    pub fn new() -> Result<Self, Error> {
        let config = Default::default();
        let gateway_client = Arc::new(
            GatewayClient::new_plain("127.0.0.1", 26500, config)
                .map_err(|e| Error::GatewayError(e))?,
        );
        Ok(Self { gateway_client })
    }

    /// Get the topology. The returned struct is similar to what is printed when running `zbctl status`.
    pub fn topology(&self) -> Result<TopologyResponse, Error> {
        let options = Default::default();
        let topology_request = Default::default();
        let grpc_response: grpc::SingleResponse<_> =
            self.gateway_client.topology(options, topology_request);
        let topology_response = grpc_response
            .wait_drop_metadata()
            .map_err(|e| Error::TopologyError(e))?;
        Ok(topology_response)
    }

    /// list the workflows
    pub fn list_workflows(&self) -> Result<Vec<WorkflowMetadata>, Error> {
        let options = Default::default();
        let list_workflows_request = Default::default();
        let grpc_response: grpc::SingleResponse<_> = self
            .gateway_client
            .list_workflows(options, list_workflows_request);
        let list_workflows_response: ListWorkflowsResponse = grpc_response
            .wait_drop_metadata()
            .map_err(|e| Error::ListWorkflowsError(e))?;
        let workflows: Vec<WorkflowMetadata> = list_workflows_response.workflows.into();
        Ok(workflows)
    }

    /// deploy a collection of workflows
    pub fn deploy_workflow(
        &self,
        workflow_requests: Vec<WorkflowRequestObject>,
    ) -> Result<DeployWorkflowResponse, Error> {
        let options = Default::default();
        let mut deploy_workflow_request = DeployWorkflowRequest::default();
        deploy_workflow_request.set_workflows(protobuf::RepeatedField::from(workflow_requests));
        let grpc_response: grpc::SingleResponse<_> = self
            .gateway_client
            .deploy_workflow(options, deploy_workflow_request);
        let deploy_workflow_response: DeployWorkflowResponse =
            grpc_response
                .wait_drop_metadata()
                .map_err(|e| Error::DeployWorkflowError(e))?;
        Ok(deploy_workflow_response)
    }

    /// create a workflow instance of latest version
    pub fn create_workflow_instance(
        &self,
        bpmn_process_id: String,
        payload: String,
    ) -> Result<CreateWorkflowInstanceResponse, Error> {
        let options = Default::default();
        let mut request = CreateWorkflowInstanceRequest::default();
        request.set_version(-1);
        request.set_bpmnProcessId(bpmn_process_id);
        request.set_payload(payload);
        let grpc_response: grpc::SingleResponse<_> = self
            .gateway_client
            .create_workflow_instance(options, request);
        let create_workflow_instance_response: CreateWorkflowInstanceResponse = grpc_response
            .wait_drop_metadata()
            .map_err(|e| Error::CreateWorkflowInstanceError(e))?;
        Ok(create_workflow_instance_response)
    }

    /// activate jobs
    pub fn activate_jobs(&self, jobs_config: &ActivateJobsConfig) -> ActivateJobs {
        ActivateJobs::new(&self.gateway_client, &jobs_config)
    }

    /// complete a job
    pub fn complete_job(&self, completed_job_data: CompletedJobData) -> CompleteJob {
        CompleteJob::new(&self.gateway_client, completed_job_data)
    }

    /// Publish a message
    pub fn publish_message(
        &self,
        name: String,
        correlation_key: String,
        time_to_live: i64,
        message_id: String,
        payload: String,
    ) -> Result<(), Error> {
        let options = Default::default();
        let mut publish_message_request = PublishMessageRequest::default();
        publish_message_request.set_payload(payload);
        publish_message_request.set_correlationKey(correlation_key);
        publish_message_request.set_messageId(message_id);
        publish_message_request.set_name(name);
        publish_message_request.set_timeToLive(time_to_live);
        let grpc_response: grpc::SingleResponse<_> = self
            .gateway_client
            .publish_message(options, publish_message_request);
        let result = grpc_response
            .wait_drop_metadata()
            .map(|_| ())
            .map_err(|e| Error::PublishMessageError(e));
        result
    }

    /// Get a worker, client must be in an `Arc`
    pub fn worker<F>(self: Arc<Client>, worker_config: WorkerConfig, f: F) -> Worker
    where
        F: Fn(i64, String) -> Option<String> + Send + 'static,
    {
        Worker::new(self, worker_config, f)
    }
}

#[cfg(test)]
mod tests {
    use crate::client::Client;

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
