# zeebe-rust

This project enables building workers for [zeebe][zeebe] in Rust.

## Todos

The GRPC commands must be wrapped with a nice Rust API. The current API is synchronous. 

- [x] Get topology
- [x] List workflows
- [x] Deploy workflows
- [x] Activate and complete job
- [x] Publish message
- [x] Create task workers API
  - [ ] Bounded number of concurrent tasks
  - [ ] Support stop workflow on panic
- [ ] Parse BPMN documents (another crate?)
- [ ] Support grpc tls
- [ ] Support any zeebe server (not just local)
- [ ] Create examples
- [x] Futurize 

## Deving

Ensure `protoc` is in your path. [Install protobufs here][protobuf].

## Attributions

This project uses the protobuf file supplied by zeebe which is licensed with the Apache 2.0 license.
The license is included in the [protobuf file][zeebe_proto].

This takes considerable influence from the [python guide published by zeebe][grpc_python] and [zeebe-client-node-js][zeebe_client_node_js] because javascript is awesome. 

[zeebe]: https://zeebe.io/
[protobuf]: https://github.com/protocolbuffers/protobuf/releases
[grpc_python]: https://zeebe.io/blog/2018/11/grpc-generating-a-zeebe-python-client/
[zeebe_client_node_js]: https://github.com/CreditSenseAU/zeebe-client-node-js
[zeebe_proto]: proto/gateway.proto