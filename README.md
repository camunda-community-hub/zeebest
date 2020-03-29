# the zeebest client [![docs.rs](https://docs.rs/zeebest/badge.svg)](https://docs.rs/zeebest) [![CircleCI](https://circleci.com/gh/xmclark/zeebest.svg?style=svg)](https://circleci.com/gh/xmclark/zeebest) [![Project Status: Abandoned – Initial development has started, but there has not yet been a stable, usable release; the project has been abandoned and the author(s) do not intend on continuing development.](https://www.repostatus.org/badges/latest/abandoned.svg)](https://www.repostatus.org/#abandoned)[![Maintainers Wanted](https://img.shields.io/badge/maintainers-wanted-blue.svg)](https://github.com/pickhardt/maintainers-wanted)

__This project is no longer maintained. New maintainers are welcome.__


This is an unofficial Rust client for [zeebe][zeebe]. Zeebe is a cool workflow orchestrator for microservices. 
Get started by by spinning up a zeebe broker locally and deploy a workflow with: 

`cargo run --package zeebest --example deploy_workflow`

## The API

This crate offers a single `Client` type that comes with a plethora of methods for interacting with the zeebe gateway.

Most methods are standard zeebe operations. The `worker` method is a bit more interesting that the other. 

Workers may activate and process jobs with a handler. A worker has a max number of concurrent jobs. The worker will do 
its best to only request jobs from the broker up to the maximum amount. Each job handler may complete or fail a job.

Workers currently will only poll the gateway manually, so you will need to use another system to process jobs periodically. 
`tokio::Interval` does a pretty good job of this.

```rust
let mut client = Client::new("127.0.0.1", 26500).unwrap();

let handler = move |activated_job| { 
    Ok(JobResult::Complete { variables: None })
};

let mut worker = zeebest::JobWorker::new(
    "rusty-worker".to_string(),
    "payment-service".to_string(),
    Duration::from_secs(10).as_secs() as _,
    2, // max number of concurrent jobs
    PanicOption::FailJobOnPanic,
    client,
    handler,
);

// returns a stream of job results
let job_stream = worker.activate_and_process_job();
```

See the individual examples for how to use the client or see the [complete order-process app example][order_process] for 
a complete solution that uses a lot of the client API. 

## Futures

Unlike some other zeebe clients, this client is `futures`-first. All methods return futures or streams. 
Futures must be executed on a futures runtime like `tokio` for anything to happen. 

This crate does not have a good out-of-the-box runtime solution, so you may rely on `tokio` or your favorite `futures` runtime. 

## Async

This crate now supports futures 0.3 (std::future). This crate does not use async-await syntax because it needs to build 
in stable rust. But because everything is futures 0.3, zeebest can be used with async-await in nightly rust projects.

## Zeebe Versions

This crate will attempt to release with [zeebe][zeebe]. This library is coupled to zeebe via the gateway protobuf file. 
This service contract will likely only change between minor patches and I believe this crate will be easiest to use
if the minor version of this crate matches zeebe. 

When zeebe stabilizes to 1.0.0 this may matter less. In the mean time, use the version of `zeebest`
that matches the minor patch version of your zeebe version e.g. 0.18.x. 

## Deving

Ensure `protoc` is in your path. [Install protobufs here][protobuf]. Many of the examples require running a
zeebe broker locally. A broker may be started easily with the [zeebe docker-compose project][docker_compose].


## Contribute?

Make an issue and start a discussion! Or just submit PRs! You probably know more about this than I do.

## Attributions

This project uses the protobuf file supplied by zeebe which is licensed with the Apache 2.0 license.
The license is included in the [protobuf file][zeebe_proto].

This takes considerable influence from the [python guide published by zeebe][grpc_python], the 
[zeebe-client-node-js][zeebe_client_node_js] because javascript is awesome, and the [java client][java_client] for 
the completeness. 

[zeebe]: https://zeebe.io/
[protobuf]: https://github.com/protocolbuffers/protobuf/releases
[grpc_python]: https://zeebe.io/blog/2018/11/grpc-generating-a-zeebe-python-client/
[zeebe_client_node_js]: https://github.com/CreditSenseAU/zeebe-client-node-js
[zeebe_proto]: proto/gateway.proto
[docker_compose]: https://github.com/zeebe-io/zeebe-docker-compose
[java_client]: https://github.com/zeebe-io/zeebe/tree/develop/clients/java/src/main/java/io/zeebe/client
[order_process]: examples/order_process_app.rs
