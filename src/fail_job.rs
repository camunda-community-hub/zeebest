use crate::gateway::FailJobRequest;
use crate::gateway_grpc::Gateway;
use crate::{gateway_grpc, Error};
use futures::Future;

#[derive(Clone, Debug)]
pub struct FailJobData {
    pub job_key: i64,
    pub retries: i32,
    pub reason: Option<String>,
}


pub(crate) fn fail_job(
    gateway_client: &gateway_grpc::GatewayClient,
    job_key: i64,
    retries: i32,
) -> impl Future<Item = (), Error = Error> {
    let request_options = Default::default();
    let mut request = FailJobRequest::default();
    request.set_jobKey(job_key);
    request.set_retries(retries);
    gateway_client
        .fail_job(request_options, request)
        .drop_metadata()
        .map(|_| ())
        .map_err(|e| Error::FailJobError(e))
}
