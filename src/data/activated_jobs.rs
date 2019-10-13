pub use crate::gateway;
use crate::{Error, ActivatedJob};
use futures::task::Context;
use std::pin::Pin;
use futures::{Poll, Stream};

/// Batched up activated jobs. Each batch corresponds to the jobs in a zeebe partition.
#[derive(Debug)]
pub struct ActivatedJobs {
    pub stream: tonic::codec::Streaming<gateway::ActivateJobsResponse>
}

impl Stream for ActivatedJobs {
    type Item = Result<ActivatedJob, Error>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Pending
    }
}
