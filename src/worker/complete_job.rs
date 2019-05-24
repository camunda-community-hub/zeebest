//use crate::gateway_grpc::*;
//use crate::{gateway, Client};
//use futures::{Async, Future};
//use std::sync::{Arc, RwLock};
//
//pub struct CompleteJob<F: Future<Item = (), Error = ()>>
//{
//    f: F,
//}
//
//impl<F: Future<Item = (), Error = ()>> CompleteJob<F>
//{
//    pub fn new(job_key: i64, client: Arc<Client>) -> Self {
//        let options = Default::default();
//        let mut complete_job_request = gateway::CompleteJobRequest::default();
//        complete_job_request.set_jobKey(job_key);
//        let f = client
//            .gateway_client
//            .complete_job(options, complete_job_request)
//            .drop_metadata()
//            .map(|_| {
//                ()
//            })
//            .map_err(|_| ());
//
//        CompleteJob {
//            f,
//        }
//    }
//}
//
//impl<F: Future<Item = (), Error = ()>> Future for CompleteJob<F>
//{
//    type Item = ();
//    type Error = ();
//
//    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
//        self.f.poll()
//    }
//}
