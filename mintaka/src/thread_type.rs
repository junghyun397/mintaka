use crate::protocol::response::{Response, ResponseSender};
use std::time::Duration;

pub trait ThreadType {

    const IS_MAIN: bool;

    fn make_response<F>(&self, produce: F) where F: FnOnce() -> Response;

    fn time_exceeded(&self) -> bool;

}

pub struct MainThread<T: ResponseSender> {
    response_sender: T,
    start_time: std::time::Instant,
    running_time: Duration,
}

impl<T: ResponseSender> MainThread<T> {

    pub fn new(
        response_sender: T,
        start_time: std::time::Instant,
        running_time: Duration,
    ) -> Self {
        Self {
            response_sender,
            start_time,
            running_time,
        }
    }

}

impl<T: ResponseSender> ThreadType for MainThread<T> {
    const IS_MAIN: bool = true;

    fn make_response<F>(&self, produce: F) where F: FnOnce() -> Response {
        let response = produce();
        self.response_sender.response(response);
    }

    fn time_exceeded(&self) -> bool {
        self.start_time.elapsed() >= self.running_time
    }
}

#[derive(Clone)]
pub struct WorkerThread;

impl ThreadType for WorkerThread {
    const IS_MAIN: bool = false;

    fn make_response<F>(&self, _action: F) where F: FnOnce() -> Response { }

    fn time_exceeded(&self) -> bool {
        false
    }
}
