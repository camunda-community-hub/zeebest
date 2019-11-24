//use futures::{Future, FutureExt};
//use std::panic;
//use std::panic::AssertUnwindSafe;
//use std::pin::Pin;
//use std::sync::Arc;
//
//pub struct JobHandler {
//    job_handler: Arc<
//        dyn Fn(crate::ActivatedJob) -> Pin<Box<dyn Future<Output = crate::JobResult> + Send>>
//            + Send
//            + Sync,
//    >,
//}
//
//impl JobHandler {
//    pub fn process_job(
//        &self,
//        activated_job: crate::ActivatedJob,
//    ) -> Pin<Box<dyn Future<Output = Result<crate::JobResult, ()>> + Send>> {
//        let job_handler = self.job_handler.clone();
//        let result = panic::catch_unwind(AssertUnwindSafe(|| (job_handler)(activated_job)));
//        match result {
//            Err(_) => futures::future::err(()).boxed(),
//            Ok(f) => AssertUnwindSafe(f)
//                .catch_unwind()
//                .then(|r| match r {
//                    Err(_) => futures::future::err(()),
//                    Ok(jr) => futures::future::ok(jr),
//                })
//                .boxed(),
//        }
//    }
//
//    pub fn new(
//        job_handler: Arc<
//            dyn Fn(crate::ActivatedJob) -> Pin<Box<dyn Future<Output = crate::JobResult> + Send>>
//                + Send
//                + Sync,
//        >,
//    ) -> Self {
//        Self { job_handler }
//    }
//}
