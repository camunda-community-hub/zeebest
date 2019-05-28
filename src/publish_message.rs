use crate::gateway;
use crate::gateway_grpc;
use crate::gateway_grpc::Gateway;
use crate::Error;
use futures::Future;
use serde::Serialize;

fn publish_message_with_optional_string_payload<
    S1: Into<String>,
    S2: Into<String>,
    S3: Into<String>,
    J: Serialize,
>(
    gateway_client: &gateway_grpc::GatewayClient,
    name: S1,
    correlation_key: S2,
    time_to_live: i64,
    message_id: S3,
    payload: Result<Option<String>, serde_json::Error>,
) -> impl Future<Item = (), Error = Error> + '_ {
    let name = name.into();
    let correlation_key = correlation_key.into();
    let message_id = message_id.into();
    futures::future::result(payload)
        .map_err(|e| Error::JsonError(e))
        .and_then(move |payload| {
            let options = Default::default();
            let mut publish_message_request = gateway::PublishMessageRequest::default();
            if let Some(payload) = payload {
                publish_message_request.set_payload(payload);
            }
            publish_message_request.set_correlationKey(correlation_key);
            publish_message_request.set_messageId(message_id);
            publish_message_request.set_name(name);
            publish_message_request.set_timeToLive(time_to_live);
            let grpc_response: grpc::SingleResponse<_> =
                gateway_client.publish_message(options, publish_message_request);
            let result = grpc_response
                .drop_metadata()
                .map(|_| ())
                .map_err(|e| Error::PublishMessageError(e));
            result
        })
}

pub(crate) fn publish_message_with_serializable_payload<
    'a,
    S1: Into<String> + 'a,
    S2: Into<String> + 'a,
    S3: Into<String> + 'a,
    J: Serialize + 'a,
>(
    gateway_client: &'a gateway_grpc::GatewayClient,
    name: S1,
    correlation_key: S2,
    time_to_live: i64,
    message_id: S3,
    payload: J,
) -> impl Future<Item = (), Error = Error> + 'a {
    let payload = serde_json::to_string(&payload).map(Some);
    publish_message_with_optional_string_payload::<_, _, _, J>(
        gateway_client,
        name,
        correlation_key,
        time_to_live,
        message_id,
        payload,
    )
}

pub(crate) fn publish_message_with_no_payload<
    'a,
    S1: Into<String> + 'a,
    S2: Into<String> + 'a,
    S3: Into<String> + 'a,
>(
    gateway_client: &'a gateway_grpc::GatewayClient,
    name: S1,
    correlation_key: S2,
    time_to_live: i64,
    message_id: S3,
) -> impl Future<Item = (), Error = Error> + 'a {
    publish_message_with_optional_string_payload::<_, _, _, ()>(
        gateway_client,
        name,
        correlation_key,
        time_to_live,
        message_id,
        Ok(None),
    )
}
