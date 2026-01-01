use crate::protocol::response::{Response, ResponseSender};
use crate::utils::time::MonotonicClock;
use rusty_renju::memo::hash_key::HashKey;
use std::time::Duration;

pub trait ThreadType {

    const IS_MAIN: bool;

    fn make_response(&self, response: Response);

    fn time_exceeded(&self) -> bool;

    fn position_hash(&self) -> HashKey;

}

pub struct MainThread<CLK: MonotonicClock, T: ResponseSender> {
    position_hash: HashKey,
    response_sender: T,
    start_time: CLK,
    running_time: Option<Duration>,
}

impl<CLK: MonotonicClock, T: ResponseSender> MainThread<CLK, T> {

    pub fn new(
        position_hash: HashKey,
        response_sender: T,
        start_time: CLK,
        running_time: Option<Duration>,
    ) -> Self {
        Self {
            position_hash,
            response_sender,
            start_time,
            running_time,
        }
    }

}

impl<CLK: MonotonicClock, T: ResponseSender> ThreadType for MainThread<CLK, T> {
    const IS_MAIN: bool = true;

    fn make_response(&self, response: Response) {
        self.response_sender.response(response);
    }

    fn time_exceeded(&self) -> bool {
        self.running_time.is_some_and(|running_time| self.start_time.elapsed() >= running_time)
    }

    fn position_hash(&self) -> HashKey {
        self.position_hash
    }
}

#[derive(Clone)]
pub struct WorkerThread;

impl ThreadType for WorkerThread {
    const IS_MAIN: bool = false;

    fn make_response(&self, _response: Response) {}

    fn time_exceeded(&self) -> bool {
        false
    }

    fn position_hash(&self) -> HashKey {
        unreachable!();
    }
}
