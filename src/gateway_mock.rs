use crate::{gateway, gateway_grpc};
use futures::Future;

pub struct MockGatewayClient {
    pub jobs: Vec<i64>,
}

impl gateway_grpc::Gateway for MockGatewayClient {
    fn topology(
        &self,
        _o: grpc::RequestOptions,
        _p: gateway::TopologyRequest,
    ) -> grpc::SingleResponse<gateway::TopologyResponse> {
        unimplemented!()
    }

    fn deploy_workflow(
        &self,
        _o: grpc::RequestOptions,
        _p: gateway::DeployWorkflowRequest,
    ) -> grpc::SingleResponse<gateway::DeployWorkflowResponse> {
        unimplemented!()
    }

    fn publish_message(
        &self,
        _o: grpc::RequestOptions,
        _p: gateway::PublishMessageRequest,
    ) -> grpc::SingleResponse<gateway::PublishMessageResponse> {
        unimplemented!()
    }

    fn update_job_retries(
        &self,
        _o: grpc::RequestOptions,
        _p: gateway::UpdateJobRetriesRequest,
    ) -> grpc::SingleResponse<gateway::UpdateJobRetriesResponse> {
        unimplemented!()
    }

    fn fail_job(
        &self,
        _o: grpc::RequestOptions,
        _p: gateway::FailJobRequest,
    ) -> grpc::SingleResponse<gateway::FailJobResponse> {
        let item: Box<
            dyn Future<Item = (gateway::FailJobResponse, grpc::Metadata), Error = grpc::Error>
                + Send
                + 'static,
        > = Box::new(futures::future::ok((
            gateway::FailJobResponse::default(),
            grpc::Metadata::default(),
        )));
        grpc::SingleResponse::new(futures::future::ok((grpc::Metadata::default(), item)))
    }

    fn complete_job(
        &self,
        _o: grpc::RequestOptions,
        _p: gateway::CompleteJobRequest,
    ) -> grpc::SingleResponse<gateway::CompleteJobResponse> {
        let item: Box<dyn Future<Item = _, Error = _> + Send + 'static> = Box::new(
            futures::future::ok((gateway::CompleteJobResponse::default(), Default::default())),
        );
        grpc::SingleResponse::new(futures::future::ok((Default::default(), item)))
    }

    fn create_workflow_instance(
        &self,
        _o: grpc::RequestOptions,
        _p: gateway::CreateWorkflowInstanceRequest,
    ) -> grpc::SingleResponse<gateway::CreateWorkflowInstanceResponse> {
        unimplemented!()
    }

    fn cancel_workflow_instance(
        &self,
        _o: grpc::RequestOptions,
        _p: gateway::CancelWorkflowInstanceRequest,
    ) -> grpc::SingleResponse<gateway::CancelWorkflowInstanceResponse> {
        unimplemented!()
    }

    fn activate_jobs(
        &self,
        _o: grpc::RequestOptions,
        _p: gateway::ActivateJobsRequest,
    ) -> grpc::StreamingResponse<gateway::ActivateJobsResponse> {
        let activated_jobs: Vec<_> = self
            .jobs
            .iter()
            .map(|job_key| {
                let mut activated_job = gateway::ActivatedJob::default();
                activated_job.set_retries(5);
                activated_job.set_key(*job_key);
                activated_job
            })
            .collect();
        let mut response = gateway::ActivateJobsResponse::default();
        response.set_jobs(activated_jobs.into());
        let activated_jobs = futures::stream::iter_ok(vec![response]);
        grpc::StreamingResponse::metadata_and_stream_and_trailing_metadata(
            Default::default(),
            activated_jobs,
            futures::future::ok(Default::default()),
        )
    }

    fn resolve_incident(
        &self,
        _o: grpc::RequestOptions,
        _p: gateway::ResolveIncidentRequest,
    ) -> grpc::SingleResponse<gateway::ResolveIncidentResponse> {
        unimplemented!()
    }

    fn set_variables(
        &self,
        _o: grpc::RequestOptions,
        _p: gateway::SetVariablesRequest,
    ) -> grpc::SingleResponse<gateway::SetVariablesResponse> {
        unimplemented!()
    }
}
