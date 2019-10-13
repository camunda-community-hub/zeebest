pub use crate::gateway;
pub use crate::{WorkflowId, WorkflowVersion};
use serde::Serialize;

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
                request.version = version.into();
                request.bpmn_process_id = bpmn_process_id;
            }
            WorkflowId::WorkflowKey(key) => {
                request.workflow_key = key;
            }
        }
        if let Some(variables) = self.variables {
            request.variables = variables;
        }
        request
    }
}
