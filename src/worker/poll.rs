use futures::{Async, Stream};
use std::time::{Duration, Instant};
use tokio::timer::Interval;

#[derive(Debug)]
pub struct Tick {
    instant: Instant,
}

impl Tick {
    pub fn new(instant: Instant) -> Self {
        Tick { instant }
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Polling error {}", 0)]
    PollIntervalError(tokio::timer::Error),
}

#[derive(Debug)]
pub struct Poll {
    interval: Interval,
}

impl Poll {
    pub fn new(duration: Duration) -> Self {
        let at = Instant::now();
        let interval = Interval::new(at, duration);
        Poll { interval }
    }
}

impl Stream for Poll {
    type Item = Tick;
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        let instant = match self.interval.poll() {
            Ok(Async::Ready(Some(instant))) => instant,
            Ok(Async::Ready(None)) => return Ok(Async::Ready(None)),
            Ok(Async::NotReady) => return Ok(Async::NotReady),
            Err(e) => return Err(Error::PollIntervalError(e)),
        };
        Ok(Async::Ready(Some(Tick::new(instant))))
    }
}
