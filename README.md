# the zeebest client [![docs.rs](https://docs.rs/zeebest/badge.svg)](https://docs.rs/zeebest) [![CircleCI](https://circleci.com/gh/xmclark/zeebest.svg?style=svg)](https://circleci.com/gh/xmclark/zeebest)

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
let client = Client::new("127.0.0.1", 26500).unwrap();

let handler = move |activated_job| { 
    Ok(JobResult::Complete { variables: None })
};

let mut worker = client.worker(
    "rusty-worker",
    "payment-service",
    10000, // timeout
    2, // max number of concurrent jobs
    PanicOption::FailJobOnPanic,
    handler,
);

// returns a stream of job results
let job_stream = worker.activate_and_process_job();
```

## Futures

Unlike some other zeebe clients, this client is `futures`-first. All methods return futures or streams. 
Futures must be executed on a futures runtime like `tokio` for anything to happen. 

This crate does not have a good out-of-the-box runtime solution, so you may rely on `tokio` or your favorite `futures` runtime. 

All of the commands may be run synchronously on the current thread with `Future::wait`. I am interested in adopting
a node.js style API with sync and async methods. 

## Async

This crate **does not support** the new async syntax. When the async stuff finds its way into stable rust, I would be 
interested in upgrading the futures from 0.1 to 0.3 and supporting the async syntax. It will definitely help with readability. 

Watch https://areweasyncyet.rs/ for updates.

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
