use crate::activate_and_process_jobs::{activate_and_process_jobs, JobError, WorkerConfig};
use crate::activate_jobs::{activate_jobs, ActivateJobsConfig};
use crate::complete_job::{complete_job, CompletedJobData};
use crate::gateway;
pub use crate::gateway::{
    ActivateJobsResponse, ActivatedJob, CreateWorkflowInstanceRequest,
    CreateWorkflowInstanceResponse, DeployWorkflowRequest, DeployWorkflowResponse,
    ListWorkflowsResponse, PublishMessageRequest, TopologyResponse, WorkflowMetadata,
    WorkflowRequestObject,
};
use crate::gateway_grpc::*;
use futures::{Future, IntoFuture, Stream};
use grpc::ClientStubExt;
use std::sync::Arc;

#[cfg(feature = "timer")]
use std::time::Duration;
#[cfg(feature = "timer")]
use tokio::timer::Interval;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Gateway Error. {:?}", _0)]
    GatewayError(grpc::Error),
    #[fail(display = "Topology Error. {:?}", _0)]
    TopologyError(grpc::Error),
    #[fail(display = "List Workflows Error. {:?}", _0)]
    ListWorkflowsError(grpc::Error),
    #[fail(display = "Deploy Workflow Error. {:?}", _0)]
    DeployWorkflowError(grpc::Error),
    #[fail(display = "Create Workflow Instance Error. {:?}", _0)]
    CreateWorkflowInstanceError(grpc::Error),
    #[fail(display = "Activate Job Error. {:?}", _0)]
    ActivateJobError(grpc::Error),
    #[fail(display = "Complete Job Error. {:?}", _0)]
    CompleteJobError(grpc::Error),
    #[fail(display = "Publish Message Error. {:?}", _0)]
    PublishMessageError(grpc::Error),
    #[fail(display = "Interval Error. {:?}", _0)]
    IntervalError(tokio::timer::Error),
    #[fail(display = "Job Error. {:?}", _0)]
    JobError(JobError),
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
    pub fn topology(&self) -> impl Future<Item = TopologyResponse, Error = Error> {
        let options = Default::default();
        let topology_request = Default::default();
        let grpc_response: grpc::SingleResponse<_> =
            self.gateway_client.topology(options, topology_request);
        grpc_response
            .drop_metadata()
            .map_err(|e| Error::TopologyError(e))
    }

    /// list the workflows
    pub fn list_workflows<I>(&self) -> impl Future<Item = Vec<WorkflowMetadata>, Error = Error> {
        let options = Default::default();
        let list_workflows_request = Default::default();
        let grpc_response: grpc::SingleResponse<ListWorkflowsResponse> = self
            .gateway_client
            .list_workflows(options, list_workflows_request);
        grpc_response
            .drop_metadata()
            .map(|r| r.workflows.into_vec())
            .map_err(|e| Error::ListWorkflowsError(e))
    }

    /// deploy a collection of workflows
    pub fn deploy_workflow(
        &self,
        workflow_requests: Vec<WorkflowRequestObject>,
    ) -> impl Future<Item = DeployWorkflowResponse, Error = Error> {
        let options = Default::default();
        let mut deploy_workflow_request = DeployWorkflowRequest::default();
        deploy_workflow_request.set_workflows(protobuf::RepeatedField::from(workflow_requests));
        let grpc_response: grpc::SingleResponse<_> = self
            .gateway_client
            .deploy_workflow(options, deploy_workflow_request);
        grpc_response
            .drop_metadata()
            .map_err(|e| Error::DeployWorkflowError(e))
    }

    /// create a workflow instance of latest version
    pub fn create_workflow_instance(
        &self,
        bpmn_process_id: String,
        payload: String,
    ) -> impl Future<Item = CreateWorkflowInstanceResponse, Error = Error> {
        let options = Default::default();
        let mut request = CreateWorkflowInstanceRequest::default();
        request.set_version(-1);
        request.set_bpmnProcessId(bpmn_process_id);
        request.set_payload(payload);
        let grpc_response: grpc::SingleResponse<_> = self
            .gateway_client
            .create_workflow_instance(options, request);
        grpc_response
            .drop_metadata()
            .map_err(|e| Error::CreateWorkflowInstanceError(e))
    }

    /// activate jobs
    pub fn activate_jobs(
        &self,
        jobs_config: &ActivateJobsConfig,
    ) -> impl Stream<Item = gateway::ActivatedJob, Error = grpc::Error> + Send {
        activate_jobs(&self.gateway_client, &jobs_config)
    }

    /// complete a job
    pub fn complete_job(
        &self,
        completed_job_data: CompletedJobData,
    ) -> impl Future<Item = CompletedJobData, Error = grpc::Error> + Send {
        complete_job(&self.gateway_client, completed_job_data)
    }

    /// Publish a message
    pub fn publish_message(
        &self,
        name: String,
        correlation_key: String,
        time_to_live: i64,
        message_id: String,
        payload: String,
    ) -> impl Future<Item = (), Error = Error> {
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
            .drop_metadata()
            .map(|_| ())
            .map_err(|e| Error::PublishMessageError(e));
        result
    }

    pub fn activate_and_process_jobs<F, X>(
        &self,
        worker_config: WorkerConfig,
        f: F,
    ) -> impl Stream<Item = CompletedJobData, Error = Error>
    where
        F: Fn(ActivatedJob) -> X + Send,
        X: IntoFuture<Item = Option<String>, Error = JobError>,
    {
        let client = self.gateway_client.clone();
        let f = Arc::new(f);
        activate_and_process_jobs(client, worker_config, f)
    }

    #[cfg(feature = "timer")]
    pub fn activate_and_process_jobs_interval<F, X>(
        &self,
        duration: Duration,
        worker_config: WorkerConfig,
        f: F,
    ) -> impl Stream<Item = CompletedJobData, Error = Error>
    where
        F: Fn(ActivatedJob) -> X + Send,
        X: IntoFuture<Item = Option<String>, Error = JobError>,
    {
        let f = Arc::new(f);
        Interval::new_interval(duration)
            .map_err(|e| Error::IntervalError(e))
            .zip(futures::stream::repeat((
                self.gateway_client.clone(),
                worker_config,
                f,
            )))
            .map(|(_, (gateway_client, worker_config, f))| {
                activate_and_process_jobs(gateway_client, worker_config, f)
            })
            .flatten()
    }
}
