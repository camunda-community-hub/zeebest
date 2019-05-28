#[macro_use]
extern crate failure;

mod activate_and_process_jobs;
mod activate_jobs;
pub mod client;
mod complete_job;
mod create_workflow_instance;
pub(crate) mod gateway;
pub(crate) mod gateway_grpc;
