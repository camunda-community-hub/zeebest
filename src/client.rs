use futures::future::{Future, TryFutureExt};
use futures::stream::Stream;
use futures::{ready, StreamExt};

pub use crate::gateway;
pub use crate::gateway::client::GatewayClient;

use serde::Serialize;
use tonic::codegen::{Body, HttpBody, StdError};
use std::pin::Pin;
use futures::task::Context;
use futures::Poll;
use crate::{ActivateJobs, ActivatedJobs, DeployedWorkflows, WorkflowInstance, CreatedWorkflowInstance, Topology};

pub mod grpc {
    type Error = Box<dyn std::error::Error + Sync + Send>;
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Gateway Error. {:?}", _0)]
    GatewayError(tonic::transport::Error),
    #[fail(display = "Topology Error. {:?}", _0)]
    TopologyError(tonic::Status),
    #[fail(display = "List Workflows Error. {:?}", _0)]
    ListWorkflowsError(tonic::Status),
    #[fail(display = "Deploy Workflow Error. {:?}", _0)]
    DeployWorkflowError(tonic::Status),
    #[fail(display = "Create Workflow Instance Error. {:?}", _0)]
    CreateWorkflowInstanceError(tonic::Status),
    #[fail(display = "Activate Job Error. {:?}", _0)]
    ActivateJobError(tonic::Status),
    #[fail(display = "Complete Job Error. {:?}", _0)]
    CompleteJobError(tonic::Status),
    #[fail(display = "Publish Message Error. {:?}", _0)]
    PublishMessageError(tonic::Status),
    #[fail(display = "Fail Job Error. {:?}", _0)]
    FailJobError(tonic::Status),
    #[cfg(feature = "timer")]
    #[fail(display = "Interval Error. {:?}", _0)]
    IntervalError(tokio::timer::Error),
    #[fail(display = "Job Error: {}", _0)]
    JobError(String),
    #[fail(display = "Json Payload Serialization Error. {:?}", _0)]
    JsonError(serde_json::error::Error),
}

/// Strongly type the version. `WorkflowVersion::Latest` is translated to `-1`.
pub enum WorkflowVersion {
    Latest,
    Version(i32),
}

impl Into<i32> for WorkflowVersion {
    fn into(self) -> i32 {
        match self {
            WorkflowVersion::Latest => -1,
            WorkflowVersion::Version(v) => v,
        }
    }
}

/// The primary type for interacting with zeebe.
#[derive(Clone)]
pub struct Client {
    pub gateway_client: gateway::client::GatewayClient<tonic::transport::Channel>,
}

impl Client {
    /// Construct a new `Client` that connects to a broker with `host` and `port`.
    pub fn new(host: &str, port: u16) -> Result<Self, Error> {
        let x = GatewayClient::<tonic::transport::Channel>::connect("localhost:3000");
        let x = x
            .map_err(|e| Error::GatewayError(e))
            .map(|gateway_client| Client { gateway_client });
        x
    }

    /// Get the topology. The returned struct is similar to what is printed when running `zbctl status`.
    pub async fn topology(&mut self) -> Result<Topology, Error> {
        let request = tonic::Request::new(gateway::TopologyRequest {});
        match self.gateway_client.topology(request).await {
            Ok(tr) => Ok(tr.into_inner().into()),
            Err(e) => Err(Error::TopologyError(e)),
        }
    }

    /// deploy a single bpmn workflow
    pub async fn deploy_bpmn_workflow<S: Into<String>>(
        &mut self,
        workflow_name: S,
        workflow_definition: Vec<u8>,
    ) -> Result<DeployedWorkflows, Error> {
        // construct request
        let mut workflow_request_object = gateway::WorkflowRequestObject::default();
        workflow_request_object.name = workflow_name.into();
        workflow_request_object.definition = workflow_definition;
        workflow_request_object.r#type = gateway::workflow_request_object::ResourceType::Bpmn as i32;
        let mut deploy_workflow_request = gateway::DeployWorkflowRequest::default();
        deploy_workflow_request.workflows = vec![workflow_request_object];
        let request = tonic::Request::new(deploy_workflow_request);
        match self.gateway_client.deploy_workflow(request).await {
            Ok(dwr) =>  Ok(DeployedWorkflows::new(dwr.into_inner())),
            Err(e) => Err(Error::DeployWorkflowError(e)),
        }
    }

    /// create a workflow instance with a payload
    pub async fn create_workflow_instance(
        &mut self,
        workflow_instance: WorkflowInstance,
    ) -> Result<CreatedWorkflowInstance, Error> {
        let request = tonic::Request::new(workflow_instance.into());
        match self.gateway_client.create_workflow_instance(request).await {
            Ok(cwr) => Ok(CreatedWorkflowInstance::new(cwr.into_inner())),
            Err(e) => Err(Error::CreateWorkflowInstanceError(e)),
        }
    }

    /// activate jobs
    pub async fn activate_jobs(
        &mut self,
        jobs_config: ActivateJobs,
    ) -> Result<ActivatedJobs, Error> {
        let request = tonic::Request::new(jobs_config.into());
        match self.gateway_client.activate_jobs(request).await {
            Ok(ajr) => Ok(ActivatedJobs {
                stream: ajr.into_inner(),
            }),
            Err(e) => Err(Error::ActivateJobError(e)),
        }
    }

    // complete a job
    //    pub fn complete_job(
    //        &mut self,
    //        complete_job: CompleteJob,
    //    ) -> impl Future<Output = Result<(), Error>> + Send + '_ {
    //        let request = tonic::Request::new(complete_job.into());
    //        self.gateway_client
    //            .complete_job(request)
    //            .map_err(|e| Error::CompleteJobError(e))
    //            .map_ok(|_| ())
    //    }

    // fail a job
    //    pub fn fail_job(
    //        &mut self,
    //        job_key: i64,
    //        retries: i32,
    //        error_message: String,
    //    ) -> impl Future<Output = Result<(), Error>> + Send + '_ {
    //        let mut request = gateway::FailJobRequest::default();
    //        request.job_key = job_key;
    //        request.retries = retries;
    //        request.error_message = error_message;
    //        let request = tonic::Request::new(request);
    //        self.gateway_client
    //            .fail_job(request)
    //            .map_ok(|_| ())
    //            .map_err(|e| Error::FailJobError(e))
    //    }

    // Publish a message
    //    pub fn publish_message(
    //        &mut self,
    //        publish_message: PublishMessage,
    //    ) -> impl Future<Output = Result<(), Error>> + '_ {
    //        let request = tonic::Request::new(publish_message.into());
    //        self.gateway_client
    //            .publish_message(request)
    //            .map_err(|e| Error::PublishMessageError(e))
    //            .map_ok(|_| ())
    //    }
}




