use crate::gateway;

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
            workflow_key: cwir.workflow_key,
            bpmn_process_id: cwir.bpmn_process_id,
            version: cwir.version,
            workflow_instance_key: cwir.workflow_instance_key,
        }
    }
}
