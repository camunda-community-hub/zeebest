use crate::gateway_grpc::Gateway;
use crate::{gateway, gateway_grpc};
use futures::{Async, Stream};

pub struct ActivateJobsConfig {
    pub worker: String,
    pub job_type: String,
    pub timeout: i64,
    pub amount: i32,
}

/// A future activates jobs and flattens them to a stream of gateway::ActivatedJob
pub struct ActivateJobs {
    s: Box<Stream<Item = gateway::ActivatedJob, Error = grpc::Error> + Send>,
}

impl ActivateJobs {
    pub fn new(client: &gateway_grpc::GatewayClient, jobs_config: &ActivateJobsConfig) -> Self {
        let stream = Self::create_activated_job_stream(client, jobs_config);
        ActivateJobs { s: stream }
    }

    /// flatten the batched up `ActivatedJob`s
    fn create_activated_job_stream(
        client: &gateway_grpc::GatewayClient,
        jobs_config: &ActivateJobsConfig,
    ) -> Box<dyn Stream<Item = gateway::ActivatedJob, Error = grpc::Error> + Send> {
        Box::new(
            Self::create_activate_jobs_response_stream(client, jobs_config)
                .map(|r| futures::stream::iter_ok(r.jobs.into_iter()))
                .flatten(),
        )
    }

    fn create_activate_jobs_response_stream(
        client: &gateway_grpc::GatewayClient,
        jobs_config: &ActivateJobsConfig,
    ) -> Box<dyn Stream<Item = gateway::ActivateJobsResponse, Error = grpc::Error> + Send> {
        let request = Self::create_activate_jobs_request(jobs_config);
        let options = Default::default();
        let grpc_response: grpc::StreamingResponse<_> = client.activate_jobs(options, request);
        let grpc_stream = grpc_response.drop_metadata();
        grpc_stream
    }

    fn create_activate_jobs_request(
        jobs_config: &ActivateJobsConfig,
    ) -> gateway::ActivateJobsRequest {
        let mut activate_jobs_request = gateway::ActivateJobsRequest::default();
        activate_jobs_request.set_amount(10); // TODO: make this configurable
        activate_jobs_request.set_timeout(jobs_config.timeout);
        activate_jobs_request.set_worker(jobs_config.worker.clone());
        activate_jobs_request.set_field_type(jobs_config.job_type.clone());
        activate_jobs_request
    }
}

impl Stream for ActivateJobs {
    type Item = gateway::ActivatedJob;
    type Error = grpc::Error;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        self.s.poll()
    }
}
