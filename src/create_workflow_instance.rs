use crate::client::{CreateWorkflowInstanceResponse, Error, WorkflowVersion};
use crate::gateway;
use crate::gateway_grpc;
use crate::gateway_grpc::Gateway;
use futures::Future;
use serde::Serialize;

fn create_workflow_instance_with_optional_string_payload<S: Into<String>, J: Serialize>(
    gateway_client: &gateway_grpc::GatewayClient,
    bpmn_process_id: S,
    version: WorkflowVersion,
    payload_string: Result<Option<String>, serde_json::Error>,
) -> impl Future<Item = CreateWorkflowInstanceResponse, Error = Error> + '_ {
    let version = version.into();
    let bpmn_process_id = bpmn_process_id.into();
    futures::future::result(payload_string)
        .map_err(|e| Error::JsonError(e))
        .and_then(move |payload| {
            let options = Default::default();
            let mut request = gateway::CreateWorkflowInstanceRequest::default();
            request.set_version(version);
            request.set_bpmnProcessId(bpmn_process_id);
            if let Some(payload) = payload {
                request.set_payload(payload);
            }
            let grpc_response: grpc::SingleResponse<_> =
                gateway_client.create_workflow_instance(options, request);
            grpc_response
                .drop_metadata()
                .map_err(|e| Error::CreateWorkflowInstanceError(e))
        })
}

pub(crate) fn create_workflow_instance_with_serializable_payload<
    'a,
    S: Into<String> + 'a,
    J: Serialize + 'a,
>(
    gateway_client: &'a gateway_grpc::GatewayClient,
    bpmn_process_id: S,
    version: WorkflowVersion,
    value: J,
) -> impl Future<Item = CreateWorkflowInstanceResponse, Error = Error> + 'a {
    let result = serde_json::to_string(&value).map(Some);
    create_workflow_instance_with_optional_string_payload::<S, J>(
        gateway_client,
        bpmn_process_id,
        version,
        result,
    )
}

pub(crate) fn create_workflow_instance_with_no_payload<'a, S: Into<String> + 'a>(
    gateway_client: &'a gateway_grpc::GatewayClient,
    bpmn_process_id: S,
    version: WorkflowVersion,
) -> impl Future<Item = CreateWorkflowInstanceResponse, Error = Error> + 'a {
    create_workflow_instance_with_optional_string_payload::<S, ()>(
        gateway_client,
        bpmn_process_id,
        version,
        Ok(None),
    )
}
