// For a more complete documentation, refer to Zeebe documentation at:
// https://docs.zeebe.io/grpc/reference.html

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActivateJobsRequest {
    /// the job type, as defined in the BPMN process (e.g. <zeebe:taskDefinition
    /// type="payment-service" />)
    #[prost(string, tag = "1")]
    pub r#type: std::string::String,
    /// the name of the worker activating the jobs, mostly used for logging purposes
    #[prost(string, tag = "2")]
    pub worker: std::string::String,
    /// a job returned after this call will not be activated by another call until the
    /// timeout has been reached
    #[prost(int64, tag = "3")]
    pub timeout: i64,
    /// the maximum jobs to activate by this request
    #[prost(int32, tag = "4")]
    pub max_jobs_to_activate: i32,
    /// a list of variables to fetch as the job variables; if empty, all visible variables at
    /// the time of activation for the scope of the job will be returned
    #[prost(string, repeated, tag = "5")]
    pub fetch_variable: ::std::vec::Vec<std::string::String>,
    /// The request will be completed when at least one job is activated or after the requestTimeout.
    /// if the requestTimeout = 0, a default timeout is used.
    /// if the requestTimeout < 0, long polling is disabled and the request is completed immediately, even when no job is activated.
    #[prost(int64, tag = "6")]
    pub request_timeout: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActivateJobsResponse {
    /// list of activated jobs
    #[prost(message, repeated, tag = "1")]
    pub jobs: ::std::vec::Vec<ActivatedJob>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActivatedJob {
    /// the key, a unique identifier for the job
    #[prost(int64, tag = "1")]
    pub key: i64,
    /// the type of the job (should match what was requested)
    #[prost(string, tag = "2")]
    pub r#type: std::string::String,
    /// the job's workflow instance key
    #[prost(int64, tag = "3")]
    pub workflow_instance_key: i64,
    /// the bpmn process ID of the job workflow definition
    #[prost(string, tag = "4")]
    pub bpmn_process_id: std::string::String,
    /// the version of the job workflow definition
    #[prost(int32, tag = "5")]
    pub workflow_definition_version: i32,
    /// the key of the job workflow definition
    #[prost(int64, tag = "6")]
    pub workflow_key: i64,
    /// the associated task element ID
    #[prost(string, tag = "7")]
    pub element_id: std::string::String,
    /// the unique key identifying the associated task, unique within the scope of the
    /// workflow instance
    #[prost(int64, tag = "8")]
    pub element_instance_key: i64,
    /// a set of custom headers defined during modelling; returned as a serialized
    /// JSON document
    #[prost(string, tag = "9")]
    pub custom_headers: std::string::String,
    /// the name of the worker which activated this job
    #[prost(string, tag = "10")]
    pub worker: std::string::String,
    /// the amount of retries left to this job (should always be positive)
    #[prost(int32, tag = "11")]
    pub retries: i32,
    /// when the job can be activated again, sent as a UNIX epoch timestamp
    #[prost(int64, tag = "12")]
    pub deadline: i64,
    /// JSON document, computed at activation time, consisting of all visible variables to
    /// the task scope
    #[prost(string, tag = "13")]
    pub variables: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CancelWorkflowInstanceRequest {
    /// the workflow instance key (as, for example, obtained from
    /// CreateWorkflowInstanceResponse)
    #[prost(int64, tag = "1")]
    pub workflow_instance_key: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CancelWorkflowInstanceResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompleteJobRequest {
    /// the unique job identifier, as obtained from ActivateJobsResponse
    #[prost(int64, tag = "1")]
    pub job_key: i64,
    /// a JSON document representing the variables in the current task scope
    #[prost(string, tag = "2")]
    pub variables: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompleteJobResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateWorkflowInstanceRequest {
    /// the unique key identifying the workflow definition (e.g. returned from a workflow
    /// in the DeployWorkflowResponse message)
    #[prost(int64, tag = "1")]
    pub workflow_key: i64,
    /// the BPMN process ID of the workflow definition
    #[prost(string, tag = "2")]
    pub bpmn_process_id: std::string::String,
    /// the version of the process; set to -1 to use the latest version
    #[prost(int32, tag = "3")]
    pub version: i32,
    /// JSON document that will instantiate the variables for the root variable scope of the
    /// workflow instance; it must be a JSON object, as variables will be mapped in a
    /// key-value fashion. e.g. { "a": 1, "b": 2 } will create two variables, named "a" and
    /// "b" respectively, with their associated values. [{ "a": 1, "b": 2 }] would not be a
    /// valid argument, as the root of the JSON document is an array and not an object.
    #[prost(string, tag = "4")]
    pub variables: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateWorkflowInstanceResponse {
    /// the key of the workflow definition which was used to create the workflow instance
    #[prost(int64, tag = "1")]
    pub workflow_key: i64,
    /// the BPMN process ID of the workflow definition which was used to create the workflow
    /// instance
    #[prost(string, tag = "2")]
    pub bpmn_process_id: std::string::String,
    /// the version of the workflow definition which was used to create the workflow instance
    #[prost(int32, tag = "3")]
    pub version: i32,
    /// the unique identifier of the created workflow instance; to be used wherever a request
    /// needs a workflow instance key (e.g. CancelWorkflowInstanceRequest)
    #[prost(int64, tag = "4")]
    pub workflow_instance_key: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeployWorkflowRequest {
    /// List of workflow resources to deploy
    #[prost(message, repeated, tag = "1")]
    pub workflows: ::std::vec::Vec<WorkflowRequestObject>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WorkflowRequestObject {
    /// the resource basename, e.g. myProcess.bpmn
    #[prost(string, tag = "1")]
    pub name: std::string::String,
    /// the resource type; if set to BPMN or YAML then the file extension
    /// is ignored
    #[prost(enumeration = "workflow_request_object::ResourceType", tag = "2")]
    pub r#type: i32,
    /// the process definition as a UTF8-encoded string
    #[prost(bytes, tag = "3")]
    pub definition: std::vec::Vec<u8>,
}
pub mod workflow_request_object {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum ResourceType {
        /// FILE type means the gateway will try to detect the resource type
        /// using the file extension of the name field
        File = 0,
        /// extension 'bpmn'
        Bpmn = 1,
        /// extension 'yaml'
        Yaml = 2,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeployWorkflowResponse {
    /// the unique key identifying the deployment
    #[prost(int64, tag = "1")]
    pub key: i64,
    /// a list of deployed workflows
    #[prost(message, repeated, tag = "2")]
    pub workflows: ::std::vec::Vec<WorkflowMetadata>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WorkflowMetadata {
    /// the bpmn process ID, as parsed during deployment; together with the version forms a
    /// unique identifier for a specific workflow definition
    #[prost(string, tag = "1")]
    pub bpmn_process_id: std::string::String,
    /// the assigned process version
    #[prost(int32, tag = "2")]
    pub version: i32,
    /// the assigned key, which acts as a unique identifier for this workflow
    #[prost(int64, tag = "3")]
    pub workflow_key: i64,
    /// the resource name (see: WorkflowRequestObject.name) from which this workflow was
    /// parsed
    #[prost(string, tag = "4")]
    pub resource_name: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FailJobRequest {
    /// the unique job identifier, as obtained when activating the job
    #[prost(int64, tag = "1")]
    pub job_key: i64,
    /// the amount of retries the job should have left
    #[prost(int32, tag = "2")]
    pub retries: i32,
    /// an optional message describing why the job failed
    /// this is particularly useful if a job runs out of retries and an incident is raised,
    /// as it this message can help explain why an incident was raised
    #[prost(string, tag = "3")]
    pub error_message: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FailJobResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PublishMessageRequest {
    /// the name of the message
    #[prost(string, tag = "1")]
    pub name: std::string::String,
    /// the correlation key of the message
    #[prost(string, tag = "2")]
    pub correlation_key: std::string::String,
    /// how long the message should be buffered on the broker, in milliseconds
    #[prost(int64, tag = "3")]
    pub time_to_live: i64,
    /// the unique ID of the message; can be omitted. only useful to ensure only one message
    /// with the given ID will ever be published (during its lifetime)
    #[prost(string, tag = "4")]
    pub message_id: std::string::String,
    /// the message variables as a JSON document; to be valid, the root of the document must be an
    /// object, e.g. { "a": "foo" }. [ "foo" ] would not be valid.
    #[prost(string, tag = "5")]
    pub variables: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PublishMessageResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResolveIncidentRequest {
    /// the unique ID of the incident to resolve
    #[prost(int64, tag = "1")]
    pub incident_key: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResolveIncidentResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TopologyRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TopologyResponse {
    /// list of brokers part of this cluster
    #[prost(message, repeated, tag = "1")]
    pub brokers: ::std::vec::Vec<BrokerInfo>,
    /// how many nodes are in the cluster
    #[prost(int32, tag = "2")]
    pub cluster_size: i32,
    /// how many partitions are spread across the cluster
    #[prost(int32, tag = "3")]
    pub partitions_count: i32,
    /// configured replication factor for this cluster
    #[prost(int32, tag = "4")]
    pub replication_factor: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BrokerInfo {
    /// unique (within a cluster) node ID for the broker
    #[prost(int32, tag = "1")]
    pub node_id: i32,
    /// hostname of the broker
    #[prost(string, tag = "2")]
    pub host: std::string::String,
    /// port for the broker
    #[prost(int32, tag = "3")]
    pub port: i32,
    /// list of partitions managed or replicated on this broker
    #[prost(message, repeated, tag = "4")]
    pub partitions: ::std::vec::Vec<Partition>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Partition {
    /// the unique ID of this partition
    #[prost(int32, tag = "1")]
    pub partition_id: i32,
    /// the role of the broker for this partition
    #[prost(enumeration = "partition::PartitionBrokerRole", tag = "2")]
    pub role: i32,
}
pub mod partition {
    /// Describes the Raft role of the broker for a given partition
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum PartitionBrokerRole {
        Leader = 0,
        Follower = 1,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateJobRetriesRequest {
    /// the unique job identifier, as obtained through ActivateJobs
    #[prost(int64, tag = "1")]
    pub job_key: i64,
    /// the new amount of retries for the job; must be positive
    #[prost(int32, tag = "2")]
    pub retries: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateJobRetriesResponse {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetVariablesRequest {
    /// the unique identifier of a particular element; can be the workflow instance key (as
    /// obtained during instance creation), or a given element, such as a service task (see
    /// elementInstanceKey on the job message)
    #[prost(int64, tag = "1")]
    pub element_instance_key: i64,
    /// a JSON serialized document describing variables as key value pairs; the root of the document
    /// must be an object
    #[prost(string, tag = "2")]
    pub variables: std::string::String,
    /// if true, the variables will be merged strictly into the local scope (as indicated by
    /// elementInstanceKey); this means the variables is not propagated to upper scopes.
    /// for example, let's say we have two scopes, '1' and '2', with each having effective variables as:
    /// 1 => `{ "foo" : 2 }`, and 2 => `{ "bar" : 1 }`. if we send an update request with
    /// elementInstanceKey = 2, variables `{ "foo" : 5 }`, and local is true, then scope 1 will
    /// be unchanged, and scope 2 will now be `{ "bar" : 1, "foo" 5 }`. if local was false, however,
    /// then scope 1 would be `{ "foo": 5 }`, and scope 2 would be `{ "bar" : 1 }`.
    #[prost(bool, tag = "3")]
    pub local: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetVariablesResponse {}
#[doc = r" Generated client implementations."]
pub mod client {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    pub struct GatewayClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl GatewayClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> GatewayClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + HttpBody + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as HttpBody>::Error: Into<StdError> + Send,
        <T::ResponseBody as HttpBody>::Data: Into<bytes::Bytes> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        #[doc = r" Check if the service is ready."]
        pub async fn ready(&mut self) -> Result<(), tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })
        }
        #[doc = ""]
        #[doc = "Iterates through all known partitions round-robin and activates up to the requested"]
        #[doc = "maximum and streams them back to the client as they are activated."]
        #[doc = "Errors:"]
        #[doc = "INVALID_ARGUMENT:"]
        #[doc = "- type is blank (empty string, null)"]
        #[doc = "- worker is blank (empty string, null)"]
        #[doc = "- timeout less than 1"]
        #[doc = "- maxJobsToActivate is less than 1"]
        pub async fn activate_jobs(
            &mut self,
            request: impl tonic::IntoRequest<super::ActivateJobsRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::ActivateJobsResponse>>,
            tonic::Status,
        > {
            self.ready().await?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/gateway_protocol.Gateway/ActivateJobs");
            self.inner
                .server_streaming(request.into_request(), path, codec)
                .await
        }
        #[doc = ""]
        #[doc = "Cancels a running workflow instance"]
        #[doc = "Errors:"]
        #[doc = "NOT_FOUND:"]
        #[doc = "- no workflow instance exists with the given key"]
        pub async fn cancel_workflow_instance(
            &mut self,
            request: impl tonic::IntoRequest<super::CancelWorkflowInstanceRequest>,
        ) -> Result<tonic::Response<super::CancelWorkflowInstanceResponse>, tonic::Status> {
            self.ready().await?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/gateway_protocol.Gateway/CancelWorkflowInstance",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = ""]
        #[doc = "Completes a job with the given variables, which allows completing the associated service task."]
        #[doc = "Errors:"]
        #[doc = "NOT_FOUND:"]
        #[doc = "- no job exists with the given job key. Note that since jobs are removed once completed,"]
        #[doc = "it could be that this job did exist at some point."]
        #[doc = "FAILED_PRECONDITION:"]
        #[doc = "- the job was marked as failed. In that case, the related incident must be resolved before"]
        #[doc = "the job can be activated again and completed."]
        pub async fn complete_job(
            &mut self,
            request: impl tonic::IntoRequest<super::CompleteJobRequest>,
        ) -> Result<tonic::Response<super::CompleteJobResponse>, tonic::Status> {
            self.ready().await?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/gateway_protocol.Gateway/CompleteJob");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = ""]
        #[doc = "Creates and starts an instance of the specified workflow. The workflow definition to use to"]
        #[doc = "create the instance can be specified either using its unique key (as returned by"]
        #[doc = "DeployWorkflow), or using the BPMN process ID and a version. Pass -1 as the version to use the"]
        #[doc = "latest deployed version. Note that only workflows with none start events can be started through"]
        #[doc = "this command."]
        #[doc = "Errors:"]
        #[doc = "NOT_FOUND:"]
        #[doc = "- no workflow with the given key exists (if workflowKey was given)"]
        #[doc = "- no workflow with the given process ID exists (if bpmnProcessId was given but version was -1)"]
        #[doc = "- no workflow with the given process ID and version exists (if both bpmnProcessId and version were given)"]
        #[doc = "FAILED_PRECONDITION:"]
        #[doc = "- the workflow definition does not contain a none start event; only workflows with none"]
        #[doc = "start event can be started manually."]
        #[doc = "INVALID_ARGUMENT:"]
        #[doc = "- the given variables argument is not a valid JSON document; it is expected to be a valid"]
        #[doc = "JSON document where the root node is an object."]
        pub async fn create_workflow_instance(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateWorkflowInstanceRequest>,
        ) -> Result<tonic::Response<super::CreateWorkflowInstanceResponse>, tonic::Status> {
            self.ready().await?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/gateway_protocol.Gateway/CreateWorkflowInstance",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = ""]
        #[doc = "Deploys one or more workflows to Zeebe. Note that this is an atomic call,"]
        #[doc = "i.e. either all workflows are deployed, or none of them are."]
        #[doc = "Errors:"]
        #[doc = "INVALID_ARGUMENT:"]
        #[doc = "- no resources given."]
        #[doc = "- if at least one resource is invalid. A resource is considered invalid if:"]
        #[doc = "- it is not a BPMN or YAML file (currently detected through the file extension)"]
        #[doc = "- the resource data is not deserializable (e.g. detected as BPMN, but it's broken XML)"]
        #[doc = "- the workflow is invalid (e.g. an event-based gateway has an outgoing sequence flow to a task)"]
        pub async fn deploy_workflow(
            &mut self,
            request: impl tonic::IntoRequest<super::DeployWorkflowRequest>,
        ) -> Result<tonic::Response<super::DeployWorkflowResponse>, tonic::Status> {
            self.ready().await?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/gateway_protocol.Gateway/DeployWorkflow");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = ""]
        #[doc = "Marks the job as failed; if the retries argument is positive, then the job will be immediately"]
        #[doc = "activatable again, and a worker could try again to process it. If it is zero or negative however,"]
        #[doc = "an incident will be raised, tagged with the given errorMessage, and the job will not be"]
        #[doc = "activatable until the incident is resolved."]
        #[doc = "Errors:"]
        #[doc = "NOT_FOUND:"]
        #[doc = "- no job was found with the given key"]
        #[doc = "FAILED_PRECONDITION:"]
        #[doc = "- the job was not activated"]
        #[doc = "- the job is already in a failed state, i.e. ran out of retries"]
        pub async fn fail_job(
            &mut self,
            request: impl tonic::IntoRequest<super::FailJobRequest>,
        ) -> Result<tonic::Response<super::FailJobResponse>, tonic::Status> {
            self.ready().await?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/gateway_protocol.Gateway/FailJob");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = ""]
        #[doc = "Publishes a single message. Messages are published to specific partitions computed from their"]
        #[doc = "correlation keys."]
        #[doc = "Errors:"]
        #[doc = "ALREADY_EXISTS:"]
        #[doc = "- a message with the same ID was previously published (and is still alive)"]
        pub async fn publish_message(
            &mut self,
            request: impl tonic::IntoRequest<super::PublishMessageRequest>,
        ) -> Result<tonic::Response<super::PublishMessageResponse>, tonic::Status> {
            self.ready().await?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/gateway_protocol.Gateway/PublishMessage");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = ""]
        #[doc = "Resolves a given incident. This simply marks the incident as resolved; most likely a call to"]
        #[doc = "UpdateJobRetries or SetVariables will be necessary to actually resolve the"]
        #[doc = "problem, following by this call."]
        #[doc = "Errors:"]
        #[doc = "NOT_FOUND:"]
        #[doc = "- no incident with the given key exists"]
        pub async fn resolve_incident(
            &mut self,
            request: impl tonic::IntoRequest<super::ResolveIncidentRequest>,
        ) -> Result<tonic::Response<super::ResolveIncidentResponse>, tonic::Status> {
            self.ready().await?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/gateway_protocol.Gateway/ResolveIncident");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = ""]
        #[doc = "Updates all the variables of a particular scope (e.g. workflow instance, flow element instance)"]
        #[doc = "from the given JSON document."]
        #[doc = "Errors:"]
        #[doc = "NOT_FOUND:"]
        #[doc = "- no element with the given elementInstanceKey exists"]
        #[doc = "INVALID_ARGUMENT:"]
        #[doc = "- the given variables document is not a valid JSON document; valid documents are expected to"]
        #[doc = "be JSON documents where the root node is an object."]
        pub async fn set_variables(
            &mut self,
            request: impl tonic::IntoRequest<super::SetVariablesRequest>,
        ) -> Result<tonic::Response<super::SetVariablesResponse>, tonic::Status> {
            self.ready().await?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/gateway_protocol.Gateway/SetVariables");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = ""]
        #[doc = "Obtains the current topology of the cluster the gateway is part of."]
        pub async fn topology(
            &mut self,
            request: impl tonic::IntoRequest<super::TopologyRequest>,
        ) -> Result<tonic::Response<super::TopologyResponse>, tonic::Status> {
            self.ready().await?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/gateway_protocol.Gateway/Topology");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = ""]
        #[doc = "Updates the number of retries a job has left. This is mostly useful for jobs that have run out of"]
        #[doc = "retries, should the underlying problem be solved."]
        #[doc = "Errors:"]
        #[doc = "NOT_FOUND:"]
        #[doc = "- no job exists with the given key"]
        #[doc = "INVALID_ARGUMENT:"]
        #[doc = "- retries is not greater than 0"]
        pub async fn update_job_retries(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateJobRetriesRequest>,
        ) -> Result<tonic::Response<super::UpdateJobRetriesResponse>, tonic::Status> {
            self.ready().await?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/gateway_protocol.Gateway/UpdateJobRetries");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
    impl<T: Clone> Clone for GatewayClient<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod server {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with GatewayServer."]
    #[async_trait]
    pub trait Gateway: Send + Sync + 'static {
        #[doc = "Server streaming response type for the ActivateJobs method."]
        type ActivateJobsStream: Stream<Item = Result<super::ActivateJobsResponse, tonic::Status>>
            + Send
            + Sync
            + 'static;
        #[doc = ""]
        #[doc = "Iterates through all known partitions round-robin and activates up to the requested"]
        #[doc = "maximum and streams them back to the client as they are activated."]
        #[doc = "Errors:"]
        #[doc = "INVALID_ARGUMENT:"]
        #[doc = "- type is blank (empty string, null)"]
        #[doc = "- worker is blank (empty string, null)"]
        #[doc = "- timeout less than 1"]
        #[doc = "- maxJobsToActivate is less than 1"]
        async fn activate_jobs(
            &self,
            request: tonic::Request<super::ActivateJobsRequest>,
        ) -> Result<tonic::Response<Self::ActivateJobsStream>, tonic::Status> {
            Err(tonic::Status::unimplemented("Not yet implemented"))
        }
        #[doc = ""]
        #[doc = "Cancels a running workflow instance"]
        #[doc = "Errors:"]
        #[doc = "NOT_FOUND:"]
        #[doc = "- no workflow instance exists with the given key"]
        async fn cancel_workflow_instance(
            &self,
            request: tonic::Request<super::CancelWorkflowInstanceRequest>,
        ) -> Result<tonic::Response<super::CancelWorkflowInstanceResponse>, tonic::Status> {
            Err(tonic::Status::unimplemented("Not yet implemented"))
        }
        #[doc = ""]
        #[doc = "Completes a job with the given variables, which allows completing the associated service task."]
        #[doc = "Errors:"]
        #[doc = "NOT_FOUND:"]
        #[doc = "- no job exists with the given job key. Note that since jobs are removed once completed,"]
        #[doc = "it could be that this job did exist at some point."]
        #[doc = "FAILED_PRECONDITION:"]
        #[doc = "- the job was marked as failed. In that case, the related incident must be resolved before"]
        #[doc = "the job can be activated again and completed."]
        async fn complete_job(
            &self,
            request: tonic::Request<super::CompleteJobRequest>,
        ) -> Result<tonic::Response<super::CompleteJobResponse>, tonic::Status> {
            Err(tonic::Status::unimplemented("Not yet implemented"))
        }
        #[doc = ""]
        #[doc = "Creates and starts an instance of the specified workflow. The workflow definition to use to"]
        #[doc = "create the instance can be specified either using its unique key (as returned by"]
        #[doc = "DeployWorkflow), or using the BPMN process ID and a version. Pass -1 as the version to use the"]
        #[doc = "latest deployed version. Note that only workflows with none start events can be started through"]
        #[doc = "this command."]
        #[doc = "Errors:"]
        #[doc = "NOT_FOUND:"]
        #[doc = "- no workflow with the given key exists (if workflowKey was given)"]
        #[doc = "- no workflow with the given process ID exists (if bpmnProcessId was given but version was -1)"]
        #[doc = "- no workflow with the given process ID and version exists (if both bpmnProcessId and version were given)"]
        #[doc = "FAILED_PRECONDITION:"]
        #[doc = "- the workflow definition does not contain a none start event; only workflows with none"]
        #[doc = "start event can be started manually."]
        #[doc = "INVALID_ARGUMENT:"]
        #[doc = "- the given variables argument is not a valid JSON document; it is expected to be a valid"]
        #[doc = "JSON document where the root node is an object."]
        async fn create_workflow_instance(
            &self,
            request: tonic::Request<super::CreateWorkflowInstanceRequest>,
        ) -> Result<tonic::Response<super::CreateWorkflowInstanceResponse>, tonic::Status> {
            Err(tonic::Status::unimplemented("Not yet implemented"))
        }
        #[doc = ""]
        #[doc = "Deploys one or more workflows to Zeebe. Note that this is an atomic call,"]
        #[doc = "i.e. either all workflows are deployed, or none of them are."]
        #[doc = "Errors:"]
        #[doc = "INVALID_ARGUMENT:"]
        #[doc = "- no resources given."]
        #[doc = "- if at least one resource is invalid. A resource is considered invalid if:"]
        #[doc = "- it is not a BPMN or YAML file (currently detected through the file extension)"]
        #[doc = "- the resource data is not deserializable (e.g. detected as BPMN, but it's broken XML)"]
        #[doc = "- the workflow is invalid (e.g. an event-based gateway has an outgoing sequence flow to a task)"]
        async fn deploy_workflow(
            &self,
            request: tonic::Request<super::DeployWorkflowRequest>,
        ) -> Result<tonic::Response<super::DeployWorkflowResponse>, tonic::Status> {
            Err(tonic::Status::unimplemented("Not yet implemented"))
        }
        #[doc = ""]
        #[doc = "Marks the job as failed; if the retries argument is positive, then the job will be immediately"]
        #[doc = "activatable again, and a worker could try again to process it. If it is zero or negative however,"]
        #[doc = "an incident will be raised, tagged with the given errorMessage, and the job will not be"]
        #[doc = "activatable until the incident is resolved."]
        #[doc = "Errors:"]
        #[doc = "NOT_FOUND:"]
        #[doc = "- no job was found with the given key"]
        #[doc = "FAILED_PRECONDITION:"]
        #[doc = "- the job was not activated"]
        #[doc = "- the job is already in a failed state, i.e. ran out of retries"]
        async fn fail_job(
            &self,
            request: tonic::Request<super::FailJobRequest>,
        ) -> Result<tonic::Response<super::FailJobResponse>, tonic::Status> {
            Err(tonic::Status::unimplemented("Not yet implemented"))
        }
        #[doc = ""]
        #[doc = "Publishes a single message. Messages are published to specific partitions computed from their"]
        #[doc = "correlation keys."]
        #[doc = "Errors:"]
        #[doc = "ALREADY_EXISTS:"]
        #[doc = "- a message with the same ID was previously published (and is still alive)"]
        async fn publish_message(
            &self,
            request: tonic::Request<super::PublishMessageRequest>,
        ) -> Result<tonic::Response<super::PublishMessageResponse>, tonic::Status> {
            Err(tonic::Status::unimplemented("Not yet implemented"))
        }
        #[doc = ""]
        #[doc = "Resolves a given incident. This simply marks the incident as resolved; most likely a call to"]
        #[doc = "UpdateJobRetries or SetVariables will be necessary to actually resolve the"]
        #[doc = "problem, following by this call."]
        #[doc = "Errors:"]
        #[doc = "NOT_FOUND:"]
        #[doc = "- no incident with the given key exists"]
        async fn resolve_incident(
            &self,
            request: tonic::Request<super::ResolveIncidentRequest>,
        ) -> Result<tonic::Response<super::ResolveIncidentResponse>, tonic::Status> {
            Err(tonic::Status::unimplemented("Not yet implemented"))
        }
        #[doc = ""]
        #[doc = "Updates all the variables of a particular scope (e.g. workflow instance, flow element instance)"]
        #[doc = "from the given JSON document."]
        #[doc = "Errors:"]
        #[doc = "NOT_FOUND:"]
        #[doc = "- no element with the given elementInstanceKey exists"]
        #[doc = "INVALID_ARGUMENT:"]
        #[doc = "- the given variables document is not a valid JSON document; valid documents are expected to"]
        #[doc = "be JSON documents where the root node is an object."]
        async fn set_variables(
            &self,
            request: tonic::Request<super::SetVariablesRequest>,
        ) -> Result<tonic::Response<super::SetVariablesResponse>, tonic::Status> {
            Err(tonic::Status::unimplemented("Not yet implemented"))
        }
        #[doc = ""]
        #[doc = "Obtains the current topology of the cluster the gateway is part of."]
        async fn topology(
            &self,
            request: tonic::Request<super::TopologyRequest>,
        ) -> Result<tonic::Response<super::TopologyResponse>, tonic::Status> {
            Err(tonic::Status::unimplemented("Not yet implemented"))
        }
        #[doc = ""]
        #[doc = "Updates the number of retries a job has left. This is mostly useful for jobs that have run out of"]
        #[doc = "retries, should the underlying problem be solved."]
        #[doc = "Errors:"]
        #[doc = "NOT_FOUND:"]
        #[doc = "- no job exists with the given key"]
        #[doc = "INVALID_ARGUMENT:"]
        #[doc = "- retries is not greater than 0"]
        async fn update_job_retries(
            &self,
            request: tonic::Request<super::UpdateJobRetriesRequest>,
        ) -> Result<tonic::Response<super::UpdateJobRetriesResponse>, tonic::Status> {
            Err(tonic::Status::unimplemented("Not yet implemented"))
        }
    }
    #[derive(Debug)]
    #[doc(hidden)]
    pub struct GatewayServer<T: Gateway> {
        inner: Arc<T>,
    }
    impl<T: Gateway> GatewayServer<T> {
        pub fn new(inner: T) -> Self {
            let inner = Arc::new(inner);
            Self { inner }
        }
    }
    impl<T: Gateway> Service<http::Request<HyperBody>> for GatewayServer<T> {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<HyperBody>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/gateway_protocol.Gateway/ActivateJobs" => {
                    struct ActivateJobsSvc<T: Gateway>(pub Arc<T>);
                    impl<T: Gateway>
                        tonic::server::ServerStreamingService<super::ActivateJobsRequest>
                        for ActivateJobsSvc<T>
                    {
                        type Response = super::ActivateJobsResponse;
                        type ResponseStream = T::ActivateJobsStream;
                        type Future =
                            BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ActivateJobsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { inner.activate_jobs(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ActivateJobsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec);
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/gateway_protocol.Gateway/CancelWorkflowInstance" => {
                    struct CancelWorkflowInstanceSvc<T: Gateway>(pub Arc<T>);
                    impl<T: Gateway>
                        tonic::server::UnaryService<super::CancelWorkflowInstanceRequest>
                        for CancelWorkflowInstanceSvc<T>
                    {
                        type Response = super::CancelWorkflowInstanceResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CancelWorkflowInstanceRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { inner.cancel_workflow_instance(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = CancelWorkflowInstanceSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec);
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/gateway_protocol.Gateway/CompleteJob" => {
                    struct CompleteJobSvc<T: Gateway>(pub Arc<T>);
                    impl<T: Gateway> tonic::server::UnaryService<super::CompleteJobRequest> for CompleteJobSvc<T> {
                        type Response = super::CompleteJobResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CompleteJobRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { inner.complete_job(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = CompleteJobSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec);
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/gateway_protocol.Gateway/CreateWorkflowInstance" => {
                    struct CreateWorkflowInstanceSvc<T: Gateway>(pub Arc<T>);
                    impl<T: Gateway>
                        tonic::server::UnaryService<super::CreateWorkflowInstanceRequest>
                        for CreateWorkflowInstanceSvc<T>
                    {
                        type Response = super::CreateWorkflowInstanceResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateWorkflowInstanceRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { inner.create_workflow_instance(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = CreateWorkflowInstanceSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec);
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/gateway_protocol.Gateway/DeployWorkflow" => {
                    struct DeployWorkflowSvc<T: Gateway>(pub Arc<T>);
                    impl<T: Gateway> tonic::server::UnaryService<super::DeployWorkflowRequest>
                        for DeployWorkflowSvc<T>
                    {
                        type Response = super::DeployWorkflowResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeployWorkflowRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { inner.deploy_workflow(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = DeployWorkflowSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec);
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/gateway_protocol.Gateway/FailJob" => {
                    struct FailJobSvc<T: Gateway>(pub Arc<T>);
                    impl<T: Gateway> tonic::server::UnaryService<super::FailJobRequest> for FailJobSvc<T> {
                        type Response = super::FailJobResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::FailJobRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { inner.fail_job(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = FailJobSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec);
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/gateway_protocol.Gateway/PublishMessage" => {
                    struct PublishMessageSvc<T: Gateway>(pub Arc<T>);
                    impl<T: Gateway> tonic::server::UnaryService<super::PublishMessageRequest>
                        for PublishMessageSvc<T>
                    {
                        type Response = super::PublishMessageResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::PublishMessageRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { inner.publish_message(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = PublishMessageSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec);
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/gateway_protocol.Gateway/ResolveIncident" => {
                    struct ResolveIncidentSvc<T: Gateway>(pub Arc<T>);
                    impl<T: Gateway> tonic::server::UnaryService<super::ResolveIncidentRequest>
                        for ResolveIncidentSvc<T>
                    {
                        type Response = super::ResolveIncidentResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ResolveIncidentRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { inner.resolve_incident(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ResolveIncidentSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec);
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/gateway_protocol.Gateway/SetVariables" => {
                    struct SetVariablesSvc<T: Gateway>(pub Arc<T>);
                    impl<T: Gateway> tonic::server::UnaryService<super::SetVariablesRequest> for SetVariablesSvc<T> {
                        type Response = super::SetVariablesResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SetVariablesRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { inner.set_variables(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = SetVariablesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec);
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/gateway_protocol.Gateway/Topology" => {
                    struct TopologySvc<T: Gateway>(pub Arc<T>);
                    impl<T: Gateway> tonic::server::UnaryService<super::TopologyRequest> for TopologySvc<T> {
                        type Response = super::TopologyResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TopologyRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { inner.topology(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = TopologySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec);
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/gateway_protocol.Gateway/UpdateJobRetries" => {
                    struct UpdateJobRetriesSvc<T: Gateway>(pub Arc<T>);
                    impl<T: Gateway> tonic::server::UnaryService<super::UpdateJobRetriesRequest>
                        for UpdateJobRetriesSvc<T>
                    {
                        type Response = super::UpdateJobRetriesResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateJobRetriesRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { inner.update_job_retries(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = UpdateJobRetriesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec);
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .body(tonic::body::BoxBody::empty())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: Gateway> Clone for GatewayServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: Gateway> tonic::transport::ServiceName for GatewayServer<T> {
        const NAME: &'static str = "gateway_protocol.Gateway";
    }
}
