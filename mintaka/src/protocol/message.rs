use crate::protocol::command::Command;
use rusty_renju::impl_debug_from_display;
use rusty_renju::notation::color::Color;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::sync::mpsc;

pub const CHANNEL_CLOSED_MESSAGE: &str = "sender channel closed.";

#[derive(Deserialize, Serialize)]
pub enum Message {
    Command(Command),
    Status(StatusCommand),
    Finished(GameResult),
    Launch,
}

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum StatusCommand {
    Version,
    Board,
    History,
}

#[derive(Eq, PartialEq, Serialize, Deserialize)]
pub enum GameResult {
    Win(Color),
    Draw,
    Full
}

impl Display for GameResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameResult::Win(color) => write!(f, "{color:?} win", ),
            GameResult::Draw => write!(f, "draw"),
            GameResult::Full => write!(f, "full"),
        }
    }
}

impl_debug_from_display!(GameResult);

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

}

pub struct MessageSender {
    sender: mpsc::Sender<Message>,
}

impl MessageSender {

    pub fn new(sender: mpsc::Sender<Message>) -> Self {
        Self { sender }
    }

    pub fn message(&self, message: Message) {
        self.sender.send(message).expect(CHANNEL_CLOSED_MESSAGE);
    }

}
