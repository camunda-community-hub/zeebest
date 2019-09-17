// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]


// interface

pub trait Gateway {
    fn activate_jobs(&self, o: ::grpc::RequestOptions, p: super::gateway::ActivateJobsRequest) -> ::grpc::StreamingResponse<super::gateway::ActivateJobsResponse>;

    fn cancel_workflow_instance(&self, o: ::grpc::RequestOptions, p: super::gateway::CancelWorkflowInstanceRequest) -> ::grpc::SingleResponse<super::gateway::CancelWorkflowInstanceResponse>;

    fn complete_job(&self, o: ::grpc::RequestOptions, p: super::gateway::CompleteJobRequest) -> ::grpc::SingleResponse<super::gateway::CompleteJobResponse>;

    fn create_workflow_instance(&self, o: ::grpc::RequestOptions, p: super::gateway::CreateWorkflowInstanceRequest) -> ::grpc::SingleResponse<super::gateway::CreateWorkflowInstanceResponse>;

    fn deploy_workflow(&self, o: ::grpc::RequestOptions, p: super::gateway::DeployWorkflowRequest) -> ::grpc::SingleResponse<super::gateway::DeployWorkflowResponse>;

    fn fail_job(&self, o: ::grpc::RequestOptions, p: super::gateway::FailJobRequest) -> ::grpc::SingleResponse<super::gateway::FailJobResponse>;

    fn publish_message(&self, o: ::grpc::RequestOptions, p: super::gateway::PublishMessageRequest) -> ::grpc::SingleResponse<super::gateway::PublishMessageResponse>;

    fn resolve_incident(&self, o: ::grpc::RequestOptions, p: super::gateway::ResolveIncidentRequest) -> ::grpc::SingleResponse<super::gateway::ResolveIncidentResponse>;

    fn set_variables(&self, o: ::grpc::RequestOptions, p: super::gateway::SetVariablesRequest) -> ::grpc::SingleResponse<super::gateway::SetVariablesResponse>;

    fn topology(&self, o: ::grpc::RequestOptions, p: super::gateway::TopologyRequest) -> ::grpc::SingleResponse<super::gateway::TopologyResponse>;

    fn update_job_retries(&self, o: ::grpc::RequestOptions, p: super::gateway::UpdateJobRetriesRequest) -> ::grpc::SingleResponse<super::gateway::UpdateJobRetriesResponse>;
}

// client

pub struct GatewayClient {
    grpc_client: ::std::sync::Arc<::grpc::Client>,
    method_ActivateJobs: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::gateway::ActivateJobsRequest, super::gateway::ActivateJobsResponse>>,
    method_CancelWorkflowInstance: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::gateway::CancelWorkflowInstanceRequest, super::gateway::CancelWorkflowInstanceResponse>>,
    method_CompleteJob: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::gateway::CompleteJobRequest, super::gateway::CompleteJobResponse>>,
    method_CreateWorkflowInstance: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::gateway::CreateWorkflowInstanceRequest, super::gateway::CreateWorkflowInstanceResponse>>,
    method_DeployWorkflow: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::gateway::DeployWorkflowRequest, super::gateway::DeployWorkflowResponse>>,
    method_FailJob: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::gateway::FailJobRequest, super::gateway::FailJobResponse>>,
    method_PublishMessage: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::gateway::PublishMessageRequest, super::gateway::PublishMessageResponse>>,
    method_ResolveIncident: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::gateway::ResolveIncidentRequest, super::gateway::ResolveIncidentResponse>>,
    method_SetVariables: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::gateway::SetVariablesRequest, super::gateway::SetVariablesResponse>>,
    method_Topology: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::gateway::TopologyRequest, super::gateway::TopologyResponse>>,
    method_UpdateJobRetries: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::gateway::UpdateJobRetriesRequest, super::gateway::UpdateJobRetriesResponse>>,
}

impl ::grpc::ClientStub for GatewayClient {
    fn with_client(grpc_client: ::std::sync::Arc<::grpc::Client>) -> Self {
        GatewayClient {
            grpc_client: grpc_client,
            method_ActivateJobs: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/gateway_protocol.Gateway/ActivateJobs".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::ServerStreaming,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_CancelWorkflowInstance: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/gateway_protocol.Gateway/CancelWorkflowInstance".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_CompleteJob: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/gateway_protocol.Gateway/CompleteJob".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_CreateWorkflowInstance: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/gateway_protocol.Gateway/CreateWorkflowInstance".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_DeployWorkflow: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/gateway_protocol.Gateway/DeployWorkflow".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_FailJob: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/gateway_protocol.Gateway/FailJob".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_PublishMessage: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/gateway_protocol.Gateway/PublishMessage".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ResolveIncident: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/gateway_protocol.Gateway/ResolveIncident".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_SetVariables: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/gateway_protocol.Gateway/SetVariables".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_Topology: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/gateway_protocol.Gateway/Topology".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_UpdateJobRetries: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/gateway_protocol.Gateway/UpdateJobRetries".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
        }
    }
}

impl Gateway for GatewayClient {
    fn activate_jobs(&self, o: ::grpc::RequestOptions, p: super::gateway::ActivateJobsRequest) -> ::grpc::StreamingResponse<super::gateway::ActivateJobsResponse> {
        self.grpc_client.call_server_streaming(o, p, self.method_ActivateJobs.clone())
    }

    fn cancel_workflow_instance(&self, o: ::grpc::RequestOptions, p: super::gateway::CancelWorkflowInstanceRequest) -> ::grpc::SingleResponse<super::gateway::CancelWorkflowInstanceResponse> {
        self.grpc_client.call_unary(o, p, self.method_CancelWorkflowInstance.clone())
    }

    fn complete_job(&self, o: ::grpc::RequestOptions, p: super::gateway::CompleteJobRequest) -> ::grpc::SingleResponse<super::gateway::CompleteJobResponse> {
        self.grpc_client.call_unary(o, p, self.method_CompleteJob.clone())
    }

    fn create_workflow_instance(&self, o: ::grpc::RequestOptions, p: super::gateway::CreateWorkflowInstanceRequest) -> ::grpc::SingleResponse<super::gateway::CreateWorkflowInstanceResponse> {
        self.grpc_client.call_unary(o, p, self.method_CreateWorkflowInstance.clone())
    }

    fn deploy_workflow(&self, o: ::grpc::RequestOptions, p: super::gateway::DeployWorkflowRequest) -> ::grpc::SingleResponse<super::gateway::DeployWorkflowResponse> {
        self.grpc_client.call_unary(o, p, self.method_DeployWorkflow.clone())
    }

    fn fail_job(&self, o: ::grpc::RequestOptions, p: super::gateway::FailJobRequest) -> ::grpc::SingleResponse<super::gateway::FailJobResponse> {
        self.grpc_client.call_unary(o, p, self.method_FailJob.clone())
    }

    fn publish_message(&self, o: ::grpc::RequestOptions, p: super::gateway::PublishMessageRequest) -> ::grpc::SingleResponse<super::gateway::PublishMessageResponse> {
        self.grpc_client.call_unary(o, p, self.method_PublishMessage.clone())
    }

    fn resolve_incident(&self, o: ::grpc::RequestOptions, p: super::gateway::ResolveIncidentRequest) -> ::grpc::SingleResponse<super::gateway::ResolveIncidentResponse> {
        self.grpc_client.call_unary(o, p, self.method_ResolveIncident.clone())
    }

    fn set_variables(&self, o: ::grpc::RequestOptions, p: super::gateway::SetVariablesRequest) -> ::grpc::SingleResponse<super::gateway::SetVariablesResponse> {
        self.grpc_client.call_unary(o, p, self.method_SetVariables.clone())
    }

    fn topology(&self, o: ::grpc::RequestOptions, p: super::gateway::TopologyRequest) -> ::grpc::SingleResponse<super::gateway::TopologyResponse> {
        self.grpc_client.call_unary(o, p, self.method_Topology.clone())
    }

    fn update_job_retries(&self, o: ::grpc::RequestOptions, p: super::gateway::UpdateJobRetriesRequest) -> ::grpc::SingleResponse<super::gateway::UpdateJobRetriesResponse> {
        self.grpc_client.call_unary(o, p, self.method_UpdateJobRetries.clone())
    }
}

// server

pub struct GatewayServer;


impl GatewayServer {
    pub fn new_service_def<H : Gateway + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::rt::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::rt::ServerServiceDefinition::new("/gateway_protocol.Gateway",
            vec![
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/gateway_protocol.Gateway/ActivateJobs".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::ServerStreaming,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerServerStreaming::new(move |o, p| handler_copy.activate_jobs(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/gateway_protocol.Gateway/CancelWorkflowInstance".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.cancel_workflow_instance(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/gateway_protocol.Gateway/CompleteJob".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.complete_job(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/gateway_protocol.Gateway/CreateWorkflowInstance".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.create_workflow_instance(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/gateway_protocol.Gateway/DeployWorkflow".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.deploy_workflow(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/gateway_protocol.Gateway/FailJob".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.fail_job(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/gateway_protocol.Gateway/PublishMessage".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.publish_message(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/gateway_protocol.Gateway/ResolveIncident".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.resolve_incident(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/gateway_protocol.Gateway/SetVariables".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.set_variables(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/gateway_protocol.Gateway/Topology".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.topology(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/gateway_protocol.Gateway/UpdateJobRetries".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.update_job_retries(o, p))
                    },
                ),
            ],
        )
    }
}
