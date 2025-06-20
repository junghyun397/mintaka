use crate::config::SearchLimit;
use crate::protocol::message::ResponseSender;
use crate::protocol::response::Response;

pub trait ThreadType {

    const IS_MAIN: bool;

    fn make_response<F>(&self, produce: F) where F: FnOnce() -> Response;

    fn limit_exceeded(&self, total_nodes_in_1k: usize) -> bool;

}

pub struct MainThread {
    response_sender: ResponseSender,
    start_time: std::time::Instant,
    search_limit: SearchLimit,
}

impl MainThread {

    pub fn new(
        response_sender: ResponseSender,
        start_time: std::time::Instant,
        search_limit: SearchLimit,
    ) -> Self {
        Self {
            response_sender,
            start_time,
            search_limit,
        }
    }

}

impl ThreadType for MainThread {
    const IS_MAIN: bool = true;

    fn make_response<F>(&self, produce: F) where F: FnOnce() -> Response {
        let response = produce();
        self.response_sender.response(response);
    }

    fn limit_exceeded(&self, total_nodes_in_1k: usize) -> bool {
        match self.search_limit {
            SearchLimit::Time { turn_time } =>
                self.start_time.elapsed() > turn_time,
            SearchLimit::Nodes { in_1k: finish_in_1k } =>
                total_nodes_in_1k > finish_in_1k,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct WorkerThread;

impl ThreadType for WorkerThread {
    const IS_MAIN: bool = false;

    fn make_response<F>(&self, _action: F) where F: FnOnce() -> Response { }

    fn limit_exceeded(&self, _total_nodes_in_1k: usize) -> bool {
        false
    }
}
