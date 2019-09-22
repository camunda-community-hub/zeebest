use crate::gateway;
use crate::gateway_grpc::*;
use futures::compat::{Future01CompatExt, Stream01CompatExt};
use futures::future::{Future, TryFutureExt};
use futures::stream::{Stream, TryStreamExt};
use grpc::ClientStubExt;
use std::sync::Arc;

use crate::gateway::TopologyResponse;
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
    pub gateway_client: Arc<dyn Gateway + Send + Sync>,
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
    pub fn topology(&self) -> impl Future<Output = Result<Topology, Error>> {
        self.gateway_client
            .topology(Default::default(), Default::default())
            .drop_metadata()
            .compat()
            .map_ok(|tr| Topology::new(tr))
            .map_err(|e| Error::TopologyError(e))
    }

    /// deploy a single bpmn workflow
    pub fn deploy_bpmn_workflow<S: Into<String>>(
        &self,
        workflow_name: S,
        workflow_definition: Vec<u8>,
    ) -> impl Future<Output = Result<DeployedWorkflows, Error>> {
        // construct request
        let mut workflow_request_object = gateway::WorkflowRequestObject::default();
        workflow_request_object.set_name(workflow_name.into());
        workflow_request_object.set_definition(workflow_definition);
        workflow_request_object.set_field_type(gateway::WorkflowRequestObject_ResourceType::BPMN);
        let mut deploy_workflow_request = gateway::DeployWorkflowRequest::default();
        deploy_workflow_request
            .set_workflows(protobuf::RepeatedField::from(vec![workflow_request_object]));
        // deploy the bpmn workflow
        self.gateway_client
            .deploy_workflow(Default::default(), deploy_workflow_request)
            .drop_metadata()
            .compat()
            .map_err(|e| Error::DeployWorkflowError(e))
            .map_ok(|dwr| DeployedWorkflows::new(dwr))
    }

    /// create a workflow instance with a payload
    pub fn create_workflow_instance(
        &self,
        workflow_instance: WorkflowInstance,
    ) -> impl Future<Output = Result<CreatedWorkflowInstance, Error>> {
        self.gateway_client
            .create_workflow_instance(Default::default(), workflow_instance.into())
            .drop_metadata()
            .compat()
            .map_err(|e| Error::CreateWorkflowInstanceError(e))
            .map_ok(|cwr| CreatedWorkflowInstance::new(cwr))
    }

    /// activate jobs
    pub fn activate_jobs(
        &self,
        jobs_config: ActivateJobs,
    ) -> impl Stream<Item = Result<ActivatedJobs, Error>> + Send {
        self.gateway_client
            .activate_jobs(Default::default(), jobs_config.into())
            .drop_metadata()
            .compat()
            .map_err(|e| Error::ActivateJobError(e))
            .map_ok(|ajr| ActivatedJobs::new(ajr))
    }

    /// complete a job
    pub fn complete_job(
        &self,
        complete_job: CompleteJob,
    ) -> impl Future<Output = Result<(), Error>> + Send {
        self.gateway_client
            .complete_job(Default::default(), complete_job.into())
            .drop_metadata()
            .compat()
            .map_err(|e| Error::CompleteJobError(e))
            .map_ok(|_| ())
    }

    /// fail a job
    pub fn fail_job(
        &self,
        job_key: i64,
        retries: i32,
        error_message: String,
    ) -> impl Future<Output = Result<(), Error>> + Send {
        let request_options = Default::default();
        let mut request = gateway::FailJobRequest::default();
        request.set_jobKey(job_key);
        request.set_retries(retries);
        request.set_errorMessage(error_message);
        self.gateway_client
            .fail_job(request_options, request)
            .drop_metadata()
            .compat()
            .map_ok(|_| ())
            .map_err(|e| Error::FailJobError(e))
    }

    /// Publish a message
    pub fn publish_message(
        &self,
        publish_message: PublishMessage,
    ) -> impl Future<Output = Result<(), Error>> {
        self.gateway_client
            .publish_message(Default::default(), publish_message.into())
            .drop_metadata()
            .compat()
            .map_err(|e| Error::PublishMessageError(e))
            .map_ok(|_| ())
    }
}

/// The topology of the zeebe cluster.
#[derive(Debug)]
pub struct Topology {
    pub brokers: Vec<BrokerInfo>,
}

impl Topology {
    pub fn new(topology_response: TopologyResponse) -> Topology {
        Self {
            brokers: topology_response
                .brokers
                .into_iter()
                .map(From::from)
                .collect(),
        }
    }
}

impl From<gateway::TopologyResponse> for Topology {
    fn from(tr: gateway::TopologyResponse) -> Self {
        Self {
            brokers: tr.brokers.into_iter().map(From::from).collect(),
        }
    }
}

/// Describes a zeebe broker.
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

/// Describes a partition on a broker.
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

/// Is this broker a leader or not?
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

/// Describes a collection of deployed workflows.
#[derive(Debug)]
pub struct DeployedWorkflows {
    pub key: i64,
    pub workflows: Vec<Workflow>,
}

impl DeployedWorkflows {
    pub fn new(deploy_workflow_response: gateway::DeployWorkflowResponse) -> DeployedWorkflows {
        Self {
            key: deploy_workflow_response.key,
            workflows: deploy_workflow_response
                .workflows
                .into_iter()
                .map(From::from)
                .collect(),
        }
    }
}

/// Describes a workflow deployed on zeebe.
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

/// Describes a workflow that was instantiated on zeebe.
#[derive(Debug)]
pub struct CreatedWorkflowInstance {
    workflow_key: i64,
    bpmn_process_id: String,
    version: i32,
    workflow_instance_key: i64,
}

impl CreatedWorkflowInstance {
    pub fn new(cwir: gateway::CreateWorkflowInstanceResponse) -> Self {
        Self {
            workflow_key: cwir.workflowKey,
            bpmn_process_id: cwir.bpmnProcessId,
            version: cwir.version,
            workflow_instance_key: cwir.workflowInstanceKey,
        }
    }
}

enum WorkflowId {
    BpmnProcessId(String, WorkflowVersion),
    WorkflowKey(i64),
}

/// Describes a workflow to instantiate.
pub struct WorkflowInstance {
    id: WorkflowId,
    variables: Option<String>,
}

impl WorkflowInstance {
    pub fn workflow_instance_with_bpmn_process<S: Into<String>>(
        bpmn_process_id: S,
        version: WorkflowVersion,
    ) -> Self {
        WorkflowInstance {
            id: WorkflowId::BpmnProcessId(bpmn_process_id.into(), version),
            variables: None,
        }
    }

    pub fn workflow_instance_with_workflow_key(workflow_key: i64) -> Self {
        WorkflowInstance {
            id: WorkflowId::WorkflowKey(workflow_key),
            variables: None,
        }
    }

    pub fn variables<S: Serialize>(mut self, variables: &S) -> Result<Self, serde_json::Error> {
        serde_json::to_string(variables).map(move |v| {
            self.variables = Some(v);
            self
        })
    }
}

impl Into<gateway::CreateWorkflowInstanceRequest> for WorkflowInstance {
    fn into(self) -> gateway::CreateWorkflowInstanceRequest {
        let mut request = gateway::CreateWorkflowInstanceRequest::default();
        match self.id {
            WorkflowId::BpmnProcessId(bpmn_process_id, version) => {
                request.set_version(version.into());
                request.set_bpmnProcessId(bpmn_process_id);
            }
            WorkflowId::WorkflowKey(key) => {
                request.set_workflowKey(key);
            }
        }
        if let Some(variables) = self.variables {
            request.set_variables(variables);
        }
        request
    }
}

/// A message for publishing an event on zeebe.
pub struct PublishMessage {
    name: String,
    correlation_key: String,
    time_to_live: i64,
    message_id: String,
    variables: Option<String>,
}

impl PublishMessage {
    pub fn new<S1: Into<String>, S2: Into<String>, S3: Into<String>>(
        name: S1,
        correlation_key: S2,
        time_to_live: i64,
        message_id: S3,
    ) -> Self {
        PublishMessage {
            name: name.into(),
            correlation_key: correlation_key.into(),
            time_to_live,
            message_id: message_id.into(),
            variables: None,
        }
    }

    pub fn variables<S: Serialize>(mut self, variables: &S) -> Result<Self, Error> {
        serde_json::to_string(variables)
            .map_err(|e| Error::JsonError(e))
            .map(move |v| {
                self.variables = Some(v);
                self
            })
    }
}

impl Into<gateway::PublishMessageRequest> for PublishMessage {
    fn into(self) -> gateway::PublishMessageRequest {
        let mut publish_message_request = gateway::PublishMessageRequest::default();
        if let Some(variables) = self.variables {
            publish_message_request.set_variables(variables);
        }
        publish_message_request.set_name(self.name);
        publish_message_request.set_timeToLive(self.time_to_live);
        publish_message_request.set_messageId(self.message_id);
        publish_message_request.set_correlationKey(self.correlation_key);
        publish_message_request
    }
}

/// A message for completing a zeebe job.
#[derive(Debug)]
pub struct CompleteJob {
    pub job_key: i64,
    pub variables: Option<String>,
}

impl CompleteJob {
    pub fn new(job_key: i64, variables: Option<String>) -> Self {
        Self { job_key, variables }
    }

    pub fn variables<S: Serialize>(mut self, variables: &S) -> Result<Self, Error> {
        serde_json::to_string(variables)
            .map_err(|e| Error::JsonError(e))
            .map(move |v| {
                self.variables = Some(v);
                self
            })
    }
}

impl Into<gateway::CompleteJobRequest> for CompleteJob {
    fn into(self) -> gateway::CompleteJobRequest {
        let mut complete_job_request = gateway::CompleteJobRequest::default();
        complete_job_request.set_jobKey(self.job_key);
        if let Some(variables) = self.variables {
            complete_job_request.set_variables(variables);
        }
        complete_job_request
    }
}

/// An object used to activate jobs on the broker.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ActivateJobs {
    /// the name of the worker activating the jobs, mostly used for logging purposes
    pub worker: String,
    /// the job type, as defined in the BPMN process (e.g. <zeebe:taskDefinition type="payment-service" />)
    pub job_type: String,
    /// a job returned after this call will not be activated by another call until the timeout has been reached
    pub timeout: i64,
    /// the maximum jobs to activate by this request
    pub max_jobs_to_activate: i32,
}

impl ActivateJobs {
    pub fn new<S1: Into<String>, S2: Into<String>>(
        worker: S1,
        job_type: S2,
        timeout: i64,
        max_jobs_to_activate: i32,
    ) -> Self {
        ActivateJobs {
            worker: worker.into(),
            job_type: job_type.into(),
            timeout,
            max_jobs_to_activate,
        }
    }
}

impl Into<gateway::ActivateJobsRequest> for ActivateJobs {
    fn into(self) -> gateway::ActivateJobsRequest {
        let mut activate_jobs_request = gateway::ActivateJobsRequest::default();
        activate_jobs_request.set_maxJobsToActivate(self.max_jobs_to_activate); // TODO: make this configurable
        activate_jobs_request.set_timeout(self.timeout);
        activate_jobs_request.set_worker(self.worker);
        activate_jobs_request.set_field_type(self.job_type);
        activate_jobs_request
    }
}

/// Batched up activated jobs. Each batch corresponds to the jobs in a zeebe partition.
#[derive(Debug)]
pub struct ActivatedJobs {
    pub activated_jobs: Vec<ActivatedJob>,
}

impl ActivatedJobs {
    pub fn new(ajr: gateway::ActivateJobsResponse) -> Self {
        let activated_jobs: Vec<ActivatedJob> = ajr.jobs.into_iter().map(From::from).collect();
        ActivatedJobs { activated_jobs }
    }
}

/// Describes an activate zeebe job. Use this to do work and respond with completion or failure.
#[derive(Clone, Debug)]
pub struct ActivatedJob {
    /// the key, a unique identifier for the job
    pub key: i64,
    /// the type of the job (should match what was requested)
    pub field_type: String,
    /// a set of custom headers defined during modelling; returned as a serialized JSON document
    pub custom_headers: String,
    /// the name of the worker which activated this job
    pub worker: String,
    /// the amount of retries left to this job (should always be positive)
    pub retries: i32,
    /// when the job can be activated again, sent as a UNIX epoch timestamp
    pub deadline: i64,
    /// JSON document, computed at activation time, consisting of all visible variables to the task scope
    pub variables: String,
}

impl From<gateway::ActivatedJob> for ActivatedJob {
    fn from(aj: gateway::ActivatedJob) -> Self {
        ActivatedJob {
            key: aj.key,
            variables: aj.variables,
            worker: aj.worker,
            retries: aj.retries,
            deadline: aj.deadline,
            custom_headers: aj.customHeaders,
            field_type: aj.field_type,
        }
    }
}
