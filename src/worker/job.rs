//use crate::worker::complete_job::CompleteJob;
//use crate::Client;
//use futures::Future;
//use std::sync::{Arc};
//
//fn job<F, J>(f: F) -> Job<F, J>
//where
//    F: FnMut(i64, Option<String>) -> J,
//    J: Future<Item = Option<String>, Error = ()> + Send + 'static,
//{
//    Job { f }
//}
//
//pub struct Job<F, J>
//where
//    F: FnMut(i64, Option<String>) -> J,
//    J: Future<Item = Option<String>, Error = ()> + Send + 'static,
//{
//    f: F,
//}

//impl<F, J> Job<F, J>
//where
//    F: FnMut(i64, Option<String>) -> J,
//    J: Future<Item = Option<String>, Error = ()> + Send + 'static,
//{
//    pub fn run_job(
//        &mut self,
//        job_key: i64,
//        payload: Option<String>,
//        client: Arc<Client>,
//    ) -> CompleteJob<J> {
//        let job_future = (self.f)(job_key, payload);
//        CompleteJob::new(job_key, client)
//    }
//}
