use crate::protocol::command::Command;
use crate::protocol::response::Response;
use std::sync::mpsc;

pub enum Message {
    Command(Command),
    Response(Response),
    Status(StatusCommand),
    Launch,
    Abort,
    Quit,
}

pub enum StatusCommand {
    Version,
    Board,
    History,
}

#[derive(Clone)]
pub struct ResponseSender {
    sender: mpsc::Sender<Message>,
}

const CHANNEL_CLOSED_MESSAGE: &str = "sender channel closed.";

impl ResponseSender {

    pub fn new(sender: mpsc::Sender<Message>) -> Self {
        Self { sender }
    }

    pub fn response(&self, response: Response) {
        self.sender.send(Message::Response(response)).expect(CHANNEL_CLOSED_MESSAGE);
    }

}

pub struct CommandSender {
    sender: mpsc::Sender<Message>,
}

impl CommandSender {

    pub fn new(sender: mpsc::Sender<Message>) -> Self {
        Self { sender }
    }

    pub fn command(&self, command: Command) {
        self.sender.send(Message::Command(command)).expect(CHANNEL_CLOSED_MESSAGE);
    }

    pub fn status(&self, command: StatusCommand) {
        self.sender.send(Message::Status(command)).expect(CHANNEL_CLOSED_MESSAGE);
    }

    pub fn launch(&self) {
        self.sender.send(Message::Launch).expect(CHANNEL_CLOSED_MESSAGE);
    }

    pub fn abort(&self) {
        self.sender.send(Message::Abort).expect(CHANNEL_CLOSED_MESSAGE);
    }

    pub fn quit(&self) {
        self.sender.send(Message::Quit).expect(CHANNEL_CLOSED_MESSAGE);
    }

}
