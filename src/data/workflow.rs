pub use crate::gateway;

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
            bpmn_process_id: wm.bpmn_process_id,
            version: wm.version,
            workflow_key: wm.workflow_key,
            resource_name: wm.resource_name,
        }
    }
}
