# the zeebest client

This is a Rust client for [zeebe][zeebe]. Zeebe is a cool workflow orchestrator tool for microservices. Get started by 
by starting up a zeebe broker locally and deploy a workflow with: 

`cargo run --package zeebest --example deploy_workflow`

## Todos

There are some big issues that needed to be addressed! Some are in-flight, and there is already plenty of refactoring 
to be done. My futures-foo is not fantastic.

- [ ] TESTS (I would like help mocking the grpc stuff, and with the futures testing)
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
- [x] Support any zeebe server (not just local)
- [ ] Create examples
- [x] Futurize
- [ ] Explore making the API more ergonomic for synchronous use cases (no futures)

## Deving

Ensure `protoc` is in your path. [Install protobufs here][protobuf]. Many of the examples require running a
zeebe broker locally.


## Contribute?

Make an issue and start a discussion! Or just submit PRs! You probably know more about this than I do.

## Attributions

This project uses the protobuf file supplied by zeebe which is licensed with the Apache 2.0 license.
The license is included in the [protobuf file][zeebe_proto].

This takes considerable influence from the [python guide published by zeebe][grpc_python] and [zeebe-client-node-js][zeebe_client_node_js] because javascript is awesome. 

[zeebe]: https://zeebe.io/
[protobuf]: https://github.com/protocolbuffers/protobuf/releases
[grpc_python]: https://zeebe.io/blog/2018/11/grpc-generating-a-zeebe-python-client/
[zeebe_client_node_js]: https://github.com/CreditSenseAU/zeebe-client-node-js
[zeebe_proto]: proto/gateway.proto
