use crate::gateway_grpc::Gateway;
use crate::{gateway, gateway_grpc};
use futures::Future;

#[derive(Clone, Debug)]
pub struct CompletedJobData {
    pub job_key: i64,
    pub variables: Option<String>,
}

/// Get a future representing the complete job rpc
pub(crate) fn complete_job(
    client: &gateway_grpc::GatewayClient,
    completed_job_data: CompletedJobData,
) -> impl Future<Item = CompletedJobData, Error = grpc::Error> + Send {
    let completed_job = futures::future::ok(completed_job_data.clone());
    let options = Default::default();
    let mut complete_job_request = gateway::CompleteJobRequest::default();
    complete_job_request.set_jobKey(completed_job_data.job_key);
    if let Some(variables) = completed_job_data.variables {
        complete_job_request.set_variables(variables);
    };
    client
        .complete_job(options, complete_job_request)
        .drop_metadata()
        .and_then(|_| completed_job)
}
