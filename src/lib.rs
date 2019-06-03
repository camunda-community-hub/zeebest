#[macro_use]
extern crate failure;

mod activate_and_process_jobs;
mod activate_jobs;
mod client;
mod complete_job;
mod create_workflow_instance;
mod fail_job;
mod gateway;
mod gateway_grpc;
mod job_fn;
mod publish_message;
mod worker;

pub use activate_and_process_jobs::{JobError, JobResponse, JobResult, WorkerConfig, FutureJobResponse};
pub use activate_jobs::ActivateJobsConfig;
pub use client::{
    ActivatedJob, Client, CreateWorkflowInstanceResponse, DeployWorkflowResponse, Error,
    TopologyResponse, WorkflowMetadata, WorkflowRequestObject, WorkflowVersion,
};
pub use complete_job::CompletedJobData;
pub use job_fn::{JobFn, JobFnLike, PanicOption};
pub use worker::Worker;
