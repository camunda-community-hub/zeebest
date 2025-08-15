# the zeebest client

[![Camunda Community Extension](https://img.shields.io/badge/Community%20Extension-An%20open%20source%20community%20maintained%20project-FF4700)](https://github.com/camunda-community-hub/community)
[![docs.rs](https://docs.rs/zeebest/badge.svg)](https://docs.rs/zeebest)
[![CircleCI](https://circleci.com/gh/xmclark/zeebest.svg?style=svg)](https://circleci.com/gh/xmclark/zeebest)
[![Lifecycle Status: Abandoned – Initial development has started, but there has not yet been a stable, usable release; the project has been abandoned and the author(s) do not intend on continuing development.](https://img.shields.io/badge/Lifecycle-Abandoned-lightgrey)](https://github.com/camunda-community-hub/community/blob/main/extension-lifecycle.md)
[![Maintainer Needed](https://img.shields.io/badge/Lifecycle-Needs%20Maintainer%20-ff69b4)](https://github.com/camunda-community-hub/community/blob/main/extension-lifecycle.md) ![Compatible with: Camunda Platform 8](https://img.shields.io/badge/Compatible%20with-Camunda%20Platform%208-0072Ce)

__This project is no longer maintained. New maintainers are welcome.__

Zeebest is an unofficial Rust client for [Zeebe][zeebe], a cool workflow orchestrator for microservices.

## Quick Start

### Prerequisites

Before you continue, ensure you have the following installed:

1. **Install Rust if you don't have it already**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Protocol Buffers Compiler (protoc)**
   - **macOS** (with homebrew): `brew install protobuf`
   - **Linux**: `apt install -y protobuf-compiler`
   - **Windows** (with Winget): `winget install protobuf`

   Then run `protoc --version` to ensure compiler version is 3+.

### Building the Project

1. **Clone the repository**
   ```bash
   git clone https://github.com/camunda-community-hub/zeebest
   cd zeebest
   ```

2. **Build the project**
   ```bash
   cargo build
   ```

### Running Examples

Get started by spinning up a zeebe broker locally and deploy a workflow with:

```bash
cargo run --package zeebest --example deploy_workflow

# Start a workflow instance
cargo run --package zeebest --example start_instance
```

> Ensure `protoc` is in your path. Many of the examples require running a zeebe broker locally. A broker may be started easily with the [zeebe docker-compose project][docker_compose].

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

## Futures and Async

This client is `futures`-first. All methods return futures or streams that must be executed on a futures runtime like `tokio` or use your own favorite `futures` runtime.

### Async Support

- Supports futures 0.3 (std::future)
- Does not use `async-await` syntax but **Compatible with it** (only) in nightly Rust projects

## Zeebe Versions Compatibility

This library is coupled to [zeebe][zeebe] via the gateway protobuf file. Use the version of `zeebest` that matches the minor version of your Zeebe installation.

## Contribute?

Make an issue and start a discussion! Or just submit PRs! You probably know more about this than I do.

## Attributions

This project uses the protobuf file supplied by [zeebe][zeebe] which is licensed with the Apache 2.0 license.
The license is included in the [protobuf file][zeebe_proto].

This takes considerable influence from the [python guide published by zeebe][grpc_python], the
[zeebe-client-node-js][zeebe_client_node_js], and the [java client][java_client] for
the completeness.

[zeebe]: https://zeebe.io/
[protobuf]: https://github.com/protocolbuffers/protobuf/releases
[grpc_python]: https://zeebe.io/blog/2018/11/grpc-generating-a-zeebe-python-client/
[zeebe_client_node_js]: https://github.com/CreditSenseAU/zeebe-client-node-js
[zeebe_proto]: proto/gateway.proto
[docker_compose]: https://github.com/zeebe-io/zeebe-docker-compose
[java_client]: https://github.com/zeebe-io/zeebe/tree/develop/clients/java/src/main/java/io/zeebe/client
[order_process]: examples/order_process_app.rs
