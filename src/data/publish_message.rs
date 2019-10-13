pub use crate::gateway;
use crate::Error;
use serde::Serialize;

/// A message for publishing an event on zeebe.
pub struct PublishMessage {
    name: String,
    correlation_key: String,
    time_to_live: i64,
    message_id: String,
    variables: Option<String>,
}

impl PublishMessage {
    pub fn new<S1: Into<String>, S2: Into<String>, S3: Into<String>>(
        name: S1,
        correlation_key: S2,
        time_to_live: i64,
        message_id: S3,
    ) -> Self {
        PublishMessage {
            name: name.into(),
            correlation_key: correlation_key.into(),
            time_to_live,
            message_id: message_id.into(),
            variables: None,
        }
    }

    pub fn variables<S: Serialize>(mut self, variables: &S) -> Result<Self, Error> {
        serde_json::to_string(variables)
            .map_err(|e| Error::JsonError(e))
            .map(move |v| {
                self.variables = Some(v);
                self
            })
    }
}

impl Into<gateway::PublishMessageRequest> for PublishMessage {
    fn into(self) -> gateway::PublishMessageRequest {
        let mut publish_message_request = gateway::PublishMessageRequest::default();
        if let Some(variables) = self.variables {
            publish_message_request.variables = variables;
        }
        publish_message_request.name = self.name;
        publish_message_request.time_to_live = self.time_to_live;
        publish_message_request.message_id = self.message_id;
        publish_message_request.correlation_key = self.correlation_key;
        publish_message_request
    }
}
