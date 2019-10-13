pub use crate::gateway;
use crate::{ActivatedJob, Error};
use futures::task::Context;
use futures::{Poll, Stream, StreamExt};
use std::pin::Pin;

/// Batched up activated jobs. Each batch corresponds to the jobs in a zeebe partition.
#[derive(Debug)]
pub struct ActivatedJobs {
    pub stream: tonic::codec::Streaming<gateway::ActivateJobsResponse>,
}

impl Stream for ActivatedJobs {
    type Item = Result<Vec<ActivatedJob>, Error>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.get_mut().stream.poll_next_unpin(cx) {
            Poll::Ready(x) => Poll::Ready(match x {
                Some(r) => Some(match r {
                    Ok(ajr) => Ok(ajr.jobs.into_iter().map(Into::into).collect()),
                    Err(e) => Err(Error::ActivateJobError(e)),
                }),
                None => None,
            }),
            Poll::Pending => Poll::Pending,
        }
    }
}
