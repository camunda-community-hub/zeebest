//! An unofficial [zeebe](https://zeebe.io/) client.
//!
//! This crate contains a client for interacting with a zeebe cluster. The `Client` contract contains
//! standard methods for interacting with the cluster like `topology` and `create_workflow_intstance`.
//! It also supplies a worker for smartly running a job handler function and reporting the results
//! to the zeebe gateway.
//!

#[macro_use]
extern crate failure;
mod client;
mod gateway;
mod gateway_grpc;
mod worker;
pub use client::*;
pub use worker::*;
