pub use crate::gateway;
use serde::Serialize;
use crate::Error;

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
        complete_job_request.job_key = self.job_key;
        if let Some(variables) = self.variables {
            complete_job_request.variables = variables;
        }
        complete_job_request
    }
}

