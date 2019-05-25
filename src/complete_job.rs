use crate::gateway_grpc::Gateway;
use crate::{gateway, gateway_grpc};
use futures::{Async, Future};

#[derive(Clone, Debug)]
pub struct CompletedJobData {
    pub job_key: i64,
    pub payload: Option<String>,
}

/// A future representing the complete job rpc
pub struct CompleteJob {
    f: Box<Future<Item = CompletedJobData, Error = grpc::Error> + Send>,
}

impl CompleteJob {
    pub fn new(client: &gateway_grpc::GatewayClient, completed_job_data: CompletedJobData) -> Self {
        let completed_job = futures::future::ok(completed_job_data.clone());
        let options = Default::default();
        let mut complete_job_request = gateway::CompleteJobRequest::default();
        complete_job_request.set_jobKey(completed_job_data.job_key);
        if let Some(s) = completed_job_data.payload {
            complete_job_request.set_payload(s);
        };
        let f = Box::new(
            client
                .complete_job(options, complete_job_request)
                .drop_metadata()
                .and_then(|_| completed_job),
        );
        CompleteJob { f }
    }
}

impl Future for CompleteJob {
    type Item = CompletedJobData;
    type Error = grpc::Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        self.f.poll()
    }
}
