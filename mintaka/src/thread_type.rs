use crate::protocol::message::ResponseSender;
use crate::protocol::response::Response;

pub trait ThreadType {

    const IS_MAIN: bool;

    fn make_response<F>(&self, produce: F) where F: FnOnce() -> Response;

    fn time_limit_exceeded(&self) -> bool;

}

pub struct MainThread {
    response_sender: ResponseSender,
    start_time: std::time::Instant,
    time_limit: std::time::Duration,
}

impl MainThread {

    pub fn new(
        response_channel: ResponseSender,
        start_time: std::time::Instant,
        time_limit: std::time::Duration,
    ) -> Self {
        Self {
            response_sender: response_channel,
            start_time,
            time_limit,
        }
    }

}

impl ThreadType for MainThread {
    const IS_MAIN: bool = true;

    fn make_response<F>(&self, produce: F) where F: FnOnce() -> Response {
        let response = produce();
        self.response_sender.send(response);
    }

    fn time_limit_exceeded(&self) -> bool {
        self.start_time.elapsed() > self.time_limit
    }
}

#[derive(Clone)]
pub struct WorkerThread;

impl ThreadType for WorkerThread {
    const IS_MAIN: bool = false;

    fn make_response<F>(&self, _action: F) where F: FnOnce() -> Response { }

    fn time_limit_exceeded(&self) -> bool {
        false
    }
}
