pub use crate::gateway;

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
            custom_headers: aj.custom_headers,
            field_type: aj.r#type,
        }
    }
}
