//! An unofficial [zeebe](https://zeebe.io/) client.
//!
//! This crate contains a client for interacting with a zeebe cluster. The `Client` contract contains
//! standard methods for interacting with the cluster like `topology` and `create_workflow_intstance`.
//! It also supplies a worker for smartly running a job handler function and reporting the results
//! to the zeebe gateway.
//!

#[macro_use]
extern crate failure;
#[macro_use]
extern crate tonic;
#[macro_use]
extern crate futures;
#[macro_use]
extern crate async_std;
#[macro_use]
extern crate serde_derive;

mod client;
mod data;
mod worker;

//pub mod gateway {
//    include!("../target/proto/build/gateway_protocol.rs");
//}
mod proto;
pub mod gateway {
    pub use crate::proto::gateway_protocol::*;
}

pub use client::*;
pub use data::client_data::*;
pub use worker::*;
