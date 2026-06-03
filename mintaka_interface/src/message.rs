use mintaka::config::SearchObjective;
use mintaka::protocol::command::Command;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::{MaybePos, Pos};
use std::sync::mpsc;

pub enum Message {
    Command(MessageCommand),
    Status(StatusCommand),
    Launch {
        objective: SearchObjective,
        apply: bool,
        interactive: bool,
    },
}

pub enum MessageCommand {
    Play { pos: MaybePos },
    Set { pos: Pos, color: Color },
    Undo,
    Unset { pos: Pos, color: Color },
    Raw(Command),
}

impl MessageCommand {
    pub fn into_command(self, hash: HashKey) -> Command {
        match self {
            MessageCommand::Play { pos } => Command::Play { hash, pos },
            MessageCommand::Set { pos, color } => Command::Set { hash, pos, color },
            MessageCommand::Unset { pos, color } => Command::Unset { hash, pos, color },
            MessageCommand::Undo => Command::Undo { hash },
            MessageCommand::Raw(command) => command,
        }
    }
}

pub const CHANNEL_CLOSED_MESSAGE: &str = "sender channel closed.";

pub enum StatusCommand {
    Version,
    Board { show_last_moves: bool },
    Forbid,
    History,
    Time,
}

#[derive(Clone)]
pub struct MessageSender {
    sender: mpsc::Sender<Message>,
}

impl MessageSender {
    pub fn new(sender: mpsc::Sender<Message>) -> Self {
        Self { sender }
    }

    pub fn command(&self, command: MessageCommand) {
        self.sender
            .send(Message::Command(command))
            .expect(CHANNEL_CLOSED_MESSAGE);
    }

    pub fn status(&self, command: StatusCommand) {
        self.sender
            .send(Message::Status(command))
            .expect(CHANNEL_CLOSED_MESSAGE);
    }

    pub fn launch(&self, objective: SearchObjective, apply: bool, interactive: bool) {
        self.sender
            .send(Message::Launch { objective, apply, interactive })
            .expect(CHANNEL_CLOSED_MESSAGE);
    }
}
