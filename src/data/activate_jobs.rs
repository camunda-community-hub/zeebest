use crate::gateway;

/// An object used to activate jobs on the broker.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ActivateJobs {
    /// the name of the worker activating the jobs, mostly used for logging purposes
    pub worker: String,
    /// the job type, as defined in the BPMN process (e.g. <zeebe:taskDefinition type="payment-service" />)
    pub job_type: String,
    /// a job returned after this call will not be activated by another call until the timeout has been reached
    pub timeout: i64,
    /// the maximum jobs to activate by this request
    pub max_jobs_to_activate: i32,
}

impl ActivateJobs {
    pub fn new<S1: Into<String>, S2: Into<String>>(
        worker: S1,
        job_type: S2,
        timeout: i64,
        max_jobs_to_activate: i32,
    ) -> Self {
        ActivateJobs {
            worker: worker.into(),
            job_type: job_type.into(),
            timeout,
            max_jobs_to_activate,
        }
    }
}

impl Into<gateway::ActivateJobsRequest> for ActivateJobs {
    fn into(self) -> gateway::ActivateJobsRequest {
        let mut activate_jobs_request = gateway::ActivateJobsRequest::default();
        activate_jobs_request.max_jobs_to_activate = self.max_jobs_to_activate; // TODO: make this configurable
        activate_jobs_request.timeout = self.timeout;
        activate_jobs_request.worker = self.worker;
        activate_jobs_request.r#type = self.job_type;
        activate_jobs_request
    }
}
