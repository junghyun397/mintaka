use crate::protocol::command::Command;
use crate::protocol::response::Response;

pub trait ThreadType {

    const IS_MAIN: bool;

    fn try_recv_command(&self) -> Option<Command>;

    fn make_response<F>(&self, produce: F) where F: FnOnce() -> Response;

    fn time_limit_exceeded(&self) -> bool;

}

pub struct MainThread {
    command_receiver: std::sync::mpsc::Receiver<Command>,
    response_sender: std::sync::mpsc::Sender<Response>,
    start_time: std::time::Instant,
    time_limit: std::time::Duration,
}

impl MainThread {

    pub fn new(
        command_receiver: std::sync::mpsc::Receiver<Command>,
        response_channel: std::sync::mpsc::Sender<Response>,
        start_time: std::time::Instant,
        time_limit: std::time::Duration,
    ) -> Self {
        Self {
            command_receiver,
            response_sender: response_channel,
            start_time,
            time_limit,
        }
    }

}

impl ThreadType for MainThread {
    const IS_MAIN: bool = true;

    fn try_recv_command(&self) -> Option<Command> {
        self.command_receiver.try_recv().ok()
    }

    fn make_response<F>(&self, produce: F) where F: FnOnce() -> Response {
        let response = produce();
        self.response_sender.send(response).expect("sender channel closed.");
    }

    fn time_limit_exceeded(&self) -> bool {
        self.start_time.elapsed() > self.time_limit
    }
}

pub struct WorkerThread;

impl ThreadType for WorkerThread {
    const IS_MAIN: bool = false;

    fn try_recv_command(&self) -> Option<Command> {
        None
    }

    fn make_response<F>(&self, _action: F) where F: FnOnce() -> Response { }

    fn time_limit_exceeded(&self) -> bool {
        false
    }
}
