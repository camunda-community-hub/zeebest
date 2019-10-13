use crate::WorkflowVersion;

pub enum WorkflowId {
    BpmnProcessId(String, WorkflowVersion),
    WorkflowKey(i64),
}
