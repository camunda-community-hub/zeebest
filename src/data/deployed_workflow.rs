use crate::gateway;
use crate::Workflow;

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
