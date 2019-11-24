use std::fmt::Debug;

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
    #[fail(display = "Std IO Error. {:?}", _0)]
    StdIoError(std::io::Error),
    #[fail(display = "Invalid Cloud Token.")]
    InvalidCloudToken,
    #[fail(display = "Tonic Error: {:?}", _0)]
    TonicError(tonic::transport::Error),
    #[fail(display = "Surf Error. {:?}", _0)]
    SurfError(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[fail(display = "URI Scheme (e.g. HTTPS) is required.")]
    SchemeMissingError,
    #[fail(display = "Invalid URI Scheme supplied: {:?}", _0)]
    InvalidSchemeError(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::StdIoError(e)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(e: serde_json::error::Error) -> Self {
        Error::JsonError(e)
    }
}

impl From<tonic::transport::Error> for Error {
    fn from(e: tonic::transport::Error) -> Self {
        Error::TonicError(e)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync + 'static>> for Error {
    fn from(e: Box<dyn std::error::Error + Send + Sync + 'static>) -> Self {
        Error::SurfError(e)
    }
}
