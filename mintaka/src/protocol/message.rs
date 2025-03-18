use crate::protocol::command::Command;
use crate::protocol::response::Response;
use std::sync::mpsc;

pub enum Message {
    Command(Command),
    Response(Response),
    Launch,
    Abort,
    Quit,
}

#[derive(Clone)]
pub struct ResponseSender {
    sender: mpsc::Sender<Message>,
}

const CHANNEL_CLOSED: &str = "sender channel closed.";

impl ResponseSender {

    pub fn new(sender: mpsc::Sender<Message>) -> Self {
        Self { sender }
    }

    pub fn send(&self, response: Response) {
        self.sender.send(Message::Response(response)).expect(CHANNEL_CLOSED);
    }

}

pub struct CommandSender {
    sender: mpsc::Sender<Message>,
}

impl CommandSender {

    pub fn new(sender: mpsc::Sender<Message>) -> Self {
        Self { sender }
    }

    pub fn send(&self, command: Command) {
        self.sender.send(Message::Command(command)).expect(CHANNEL_CLOSED);
    }

    pub fn launch(&self) {
        self.sender.send(Message::Launch).expect(CHANNEL_CLOSED);
    }

    pub fn abort(&self) {
        self.sender.send(Message::Abort).expect(CHANNEL_CLOSED);
    }

    pub fn quit(&self) {
        self.sender.send(Message::Quit).expect(CHANNEL_CLOSED);
    }

}
