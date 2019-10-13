#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Gateway Error. {:?}", _0)]
    GatewayError(tonic::transport::Error),
    #[fail(display = "Topology Error. {:?}", _0)]
    TopologyError(tonic::Status),
    #[fail(display = "List Workflows Error. {:?}", _0)]
    ListWorkflowsError(tonic::Status),
    #[fail(display = "Deploy Workflow Error. {:?}", _0)]
    DeployWorkflowError(tonic::Status),
    #[fail(display = "Create Workflow Instance Error. {:?}", _0)]
    CreateWorkflowInstanceError(tonic::Status),
    #[fail(display = "Activate Job Error. {:?}", _0)]
    ActivateJobError(tonic::Status),
    #[fail(display = "Complete Job Error. {:?}", _0)]
    CompleteJobError(tonic::Status),
    #[fail(display = "Publish Message Error. {:?}", _0)]
    PublishMessageError(tonic::Status),
    #[fail(display = "Fail Job Error. {:?}", _0)]
    FailJobError(tonic::Status),
    #[cfg(feature = "timer")]
    #[fail(display = "Interval Error. {:?}", _0)]
    IntervalError(tokio::timer::Error),
    #[fail(display = "Job Error: {}", _0)]
    JobError(String),
    #[fail(display = "Json Payload Serialization Error. {:?}", _0)]
    JsonError(serde_json::error::Error),
}
