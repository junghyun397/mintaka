use mintaka::config::{Config, SearchObjective};
use mintaka::protocol::command::Command;
use rusty_renju::hash_key::HashKey;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::{MaybePos, Pos};
use std::sync::mpsc;
use std::time::Duration;
use rusty_renju::utils::byte_size::ByteSize;

pub enum Message {
    Config(ConfigCommand),
    Command(MessageCommand),
    Launch {
        objective: SearchObjective,
        apply: bool,
        interactive: bool,
    },
    Status(StatusCommand),
}

pub enum ConfigCommand {
    TotalTime(Duration),
    IncrementTime(Duration),
    TurnTime(Duration),
    MaxNodes { in_1k: u32 },
    MaxDepth(u32),
    Workers(u32),
    ResizeTT(ByteSize),
}

pub enum MessageCommand {
    Play { pos: MaybePos },
    Set { pos: Pos, color: Color },
    Undo,
    Unset { pos: Pos, color: Color },
    Command(Command),
}

impl MessageCommand {
    pub fn into_command(self, config: &Config, hash: HashKey) -> Command {
        match self {
            MessageCommand::Play { pos } => Command::Play { hash, pos, draw_condition: config.draw_condition },
            MessageCommand::Set { pos, color } => Command::Set { hash, pos, color },
            MessageCommand::Unset { pos, color } => Command::Unset { hash, pos, color },
            MessageCommand::Undo => Command::Undo { hash },
            MessageCommand::Command(command) => command,
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
    
    pub fn config(&self, command: ConfigCommand) {
        self.sender
            .send(Message::Config(command))
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
