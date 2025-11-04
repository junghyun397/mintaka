use mintaka::protocol::command::Command;
use mintaka::protocol::game_result::GameResult;
use std::fmt::Display;
use std::sync::mpsc;

pub const CHANNEL_CLOSED_MESSAGE: &str = "sender channel closed.";

pub enum Message {
    Ok,
    Command(Command),
    Status(StatusCommand),
    Finished(GameResult),
    Launch,
}

pub enum StatusCommand {
    Version,
    Board,
    History,
}

#[derive(Clone)]
pub struct MessageSender {
    sender: mpsc::Sender<Message>,
}

impl MessageSender {

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

    pub fn result(&self, result: Option<GameResult>) {
        match result {
            Some(result) => self.sender.send(Message::Finished(result)).expect(CHANNEL_CLOSED_MESSAGE),
            None => self.sender.send(Message::Ok).expect(CHANNEL_CLOSED_MESSAGE),
        }
    }

}
