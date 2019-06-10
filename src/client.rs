use crate::complete_job::{complete_job, CompletedJobData};
pub use crate::gateway::{
    ActivateJobsResponse, ActivatedJob, CreateWorkflowInstanceRequest,
    CreateWorkflowInstanceResponse, DeployWorkflowRequest, DeployWorkflowResponse,
    PublishMessageRequest, TopologyResponse, WorkflowMetadata, WorkflowRequestObject,
};
use crate::gateway_grpc::*;
use futures::{Future, IntoFuture, Stream};
use grpc::ClientStubExt;
use std::sync::Arc;

use crate::create_workflow_instance::{
    create_workflow_instance_with_no_payload, create_workflow_instance_with_serializable_payload,
};
use crate::publish_message::{
    publish_message_with_no_payload, publish_message_with_serializable_payload,
};

use crate::activate_jobs::activate_jobs;
use crate::fail_job::fail_job;
use crate::worker::{JobResult, JobWorker, PanicOption};
use crate::ActivateJobsConfig;
use serde::Serialize;

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
    #[fail(display = "Fail Job Error. {:?}", _0)]
    FailJobError(grpc::Error),
    #[cfg(feature = "timer")]
    #[fail(display = "Interval Error. {:?}", _0)]
    IntervalError(tokio::timer::Error),
    #[fail(display = "Job Error: {}", _0)]
    JobError(String),
    #[fail(display = "Json Payload Serialization Error. {:?}", _0)]
    JsonError(serde_json::error::Error),
}

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

#[derive(Clone)]
pub struct Client {
    pub(crate) gateway_client: Arc<GatewayClient>,
}

impl Client {
    /// Construct a new `Client` that connects to a broker with `host` and `port`.
    pub fn new(host: &str, port: u16) -> Result<Self, Error> {
        GatewayClient::new_plain(host, port, Default::default())
            .map_err(|e| Error::GatewayError(e))
            .map(Arc::new)
            .map(|gateway_client| Client { gateway_client })
    }

    /// Get the topology. The returned struct is similar to what is printed when running `zbctl status`.
    pub fn topology(&self) -> impl Future<Item = Topology, Error = Error> {
        self.gateway_client
            .topology(Default::default(), Default::default())
            .drop_metadata()
            .map(From::from)
            .map_err(|e| Error::TopologyError(e))
    }

    /// deploy a single bpmn workflow
    pub fn deploy_bpmn_workflow<S: Into<String>>(
        &self,
        workflow_name: S,
        workflow_definition: Vec<u8>,
    ) -> impl Future<Item = DeployedWorkflows, Error = Error> {
        // construct request
        let mut workflow_request_object = WorkflowRequestObject::default();
        workflow_request_object.set_name(workflow_name.into());
        workflow_request_object.set_definition(workflow_definition);
        workflow_request_object.set_field_type(gateway::WorkflowRequestObject_ResourceType::BPMN);
        let mut deploy_workflow_request = DeployWorkflowRequest::default();
        deploy_workflow_request
            .set_workflows(protobuf::RepeatedField::from(vec![workflow_request_object]));
        // deploy the bpmn workflow
        self.gateway_client
            .deploy_workflow(Default::default(), deploy_workflow_request)
            .drop_metadata()
            .map_err(|e| Error::DeployWorkflowError(e))
            .map(From::from)
    }

    /// create a workflow instance with a payload
    pub fn create_workflow_instance<'a, S: Into<String> + 'a, J: Serialize + 'a>(
        &'a self,
        bpmn_process_id: S,
        version: WorkflowVersion,
        value: J,
    ) -> impl Future<Item = CreateWorkflowInstanceResponse, Error = Error> + 'a {
        create_workflow_instance_with_serializable_payload(
            self.gateway_client.as_ref(),
            bpmn_process_id,
            version,
            value,
        )
    }
    /// create a workflow instance with no payload
    pub fn create_workflow_instance_no_payload<'a, S: Into<String> + 'a>(
        &'a self,
        bpmn_process_id: S,
        version: WorkflowVersion,
    ) -> impl Future<Item = CreateWorkflowInstanceResponse, Error = Error> + 'a {
        create_workflow_instance_with_no_payload(
            self.gateway_client.as_ref(),
            bpmn_process_id,
            version,
        )
    }

    /// activate jobs
    pub fn activate_jobs(
        &self,
        jobs_config: ActivateJobsConfig,
    ) -> impl Stream<Item = ActivatedJob, Error = grpc::Error> + Send {
        activate_jobs(self.gateway_client.clone(), jobs_config)
    }

    /// complete a job
    pub fn complete_job(
        &self,
        completed_job_data: CompletedJobData,
    ) -> impl Future<Item = CompletedJobData, Error = grpc::Error> + Send {
        complete_job(&self.gateway_client, completed_job_data)
    }

    /// fail a job
    pub fn fail_job(
        &self,
        job_key: i64,
        retries: i32,
    ) -> impl Future<Item = (), Error = Error> + Send {
        fail_job(&self.gateway_client, job_key, retries)
    }

    /// Publish a message with a payload
    pub fn publish_message<
        'a,
        S1: Into<String> + 'a,
        S2: Into<String> + 'a,
        S3: Into<String> + 'a,
        J: Serialize + 'a,
    >(
        &'a self,
        name: S1,
        correlation_key: S2,
        time_to_live: i64,
        message_id: S3,
        payload: J,
    ) -> impl Future<Item = (), Error = Error> + 'a {
        publish_message_with_serializable_payload(
            &self.gateway_client,
            name,
            correlation_key,
            time_to_live,
            message_id,
            payload,
        )
    }

    /// Publish a message without a payload
    pub fn publish_message_no_payload<
        'a,
        S1: Into<String> + 'a,
        S2: Into<String> + 'a,
        S3: Into<String> + 'a,
    >(
        &'a self,
        name: S1,
        correlation_key: S2,
        time_to_live: i64,
        message_id: S3,
    ) -> impl Future<Item = (), Error = Error> + 'a {
        publish_message_with_no_payload(
            &self.gateway_client,
            name,
            correlation_key,
            time_to_live,
            message_id,
        )
    }

    pub fn worker<H, F, S1, S2>(
        &self,
        worker: S1,
        job_type: S2,
        timeout: i64,
        max_amount: u16,
        panic_option: PanicOption,
        handler: H,
    ) -> JobWorker<H, F>
    where
        H: Fn(ActivatedJob) -> F + std::panic::RefUnwindSafe,
        F: IntoFuture<Item = JobResult, Error = String> + std::panic::UnwindSafe,
        <F as IntoFuture>::Future: std::panic::UnwindSafe,
        S1: Into<String>,
        S2: Into<String>,
    {
        JobWorker::new(
            worker.into(),
            job_type.into(),
            timeout,
            max_amount,
            panic_option,
            self.gateway_client.clone(),
            handler,
        )
    }
}

use crate::gateway;

#[derive(Debug)]
pub struct Topology {
    pub brokers: Vec<BrokerInfo>,
}

impl From<gateway::TopologyResponse> for Topology {
    fn from(tr: gateway::TopologyResponse) -> Self {
        Self {
            brokers: tr.brokers.into_iter().map(From::from).collect(),
        }
    }
}

#[derive(Debug)]
pub struct BrokerInfo {
    pub node_id: i32,
    pub host: String,
    pub port: i32,
    pub partitions: Vec<Partition>,
}

impl From<gateway::BrokerInfo> for BrokerInfo {
    fn from(bi: gateway::BrokerInfo) -> Self {
        Self {
            node_id: bi.nodeId,
            host: bi.host,
            port: bi.port,
            partitions: bi.partitions.into_iter().map(From::from).collect(),
        }
    }
}

#[derive(Debug)]
pub struct Partition {
    pub partition_id: i32,
    pub role: BrokerRole,
}

impl From<gateway::Partition> for Partition {
    fn from(p: gateway::Partition) -> Self {
        Self {
            partition_id: p.partitionId,
            role: p.role.into(),
        }
    }
}

#[derive(Debug)]
pub enum BrokerRole {
    LEADER = 0,
    FOLLOWER = 1,
}

impl From<gateway::Partition_PartitionBrokerRole> for BrokerRole {
    fn from(pbr: gateway::Partition_PartitionBrokerRole) -> Self {
        match pbr {
            gateway::Partition_PartitionBrokerRole::FOLLOWER => BrokerRole::FOLLOWER,
            gateway::Partition_PartitionBrokerRole::LEADER => BrokerRole::LEADER,
        }
    }
}

#[derive(Debug)]
pub struct DeployedWorkflows {
    pub key: i64,
    pub workflows: Vec<Workflow>,
}

impl From<gateway::DeployWorkflowResponse> for DeployedWorkflows {
    fn from(dwr: gateway::DeployWorkflowResponse) -> Self {
        Self {
            key: dwr.key,
            workflows: dwr.workflows.into_iter().map(From::from).collect(),
        }
    }
}

#[derive(Debug)]
pub struct Workflow {
    pub bpmn_process_id: String,
    pub version: i32,
    pub workflow_key: i64,
    pub resource_name: String,
}

impl From<gateway::WorkflowMetadata> for Workflow {
    fn from(wm: gateway::WorkflowMetadata) -> Self {
        Self {
            bpmn_process_id: wm.bpmnProcessId,
            version: wm.version,
            workflow_key: wm.workflowKey,
            resource_name: wm.resourceName,
        }
    }
}
