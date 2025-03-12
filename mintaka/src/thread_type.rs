use crate::protocol::response::Response;

pub trait ThreadType {

    const IS_MAIN: bool;

    fn make_response<F>(&self, produce: F) where F: FnOnce() -> Response;

    fn time_limit_exceeded(&self) -> bool;

}

pub struct MainThread {
    pub response_channel: std::sync::mpsc::Sender<Response>,
    pub start_time: std::time::Instant,
    pub time_limit: std::time::Duration,
}

impl ThreadType for MainThread {
    const IS_MAIN: bool = true;

    fn make_response<F>(&self, produce: F) where F: FnOnce() -> Response {
        let response = produce();
        self.response_channel.send(response).expect("sender channel closed.");
    }

    fn time_limit_exceeded(&self) -> bool {
        self.start_time.elapsed() > self.time_limit
    }
}

pub struct WorkerThread;

impl ThreadType for WorkerThread {
    const IS_MAIN: bool = false;

    fn make_response<F>(&self, _action: F) where F: FnOnce() -> Response { }

    fn time_limit_exceeded(&self) -> bool {
        false
    }
}
