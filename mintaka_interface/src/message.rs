use mintaka::config::{Config, SearchObjective};
use mintaka::protocol::command::Command;
use mintaka::protocol::time::TimeUnit;
use rusty_renju::hash_key::HashKey;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::utils::byte_size::ByteSize;
use std::sync::mpsc;

pub enum Message {
    Config(ConfigCommand),
    Command(MessageCommand),
    Launch {
        objective: SearchObjective,
        apply: bool,
        print: bool,
    },
    Status(StatusCommand),
}

pub struct MessagePacket {
    pub message: Message,
    pub ack: bool,
}

pub enum ConfigCommand {
    TimeUnit(TimeUnit),
    TotalTime(Option<u64>),
    IncrementTime(u64),
    TurnTime(Option<u64>),
    MaxDepth(Option<u32>),
    Workers(Option<u32>),
    MaxMemory(Option<ByteSize>),
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
    sender: mpsc::Sender<MessagePacket>,
}

impl MessageSender {
    pub fn new(sender: mpsc::Sender<MessagePacket>) -> Self {
        Self { sender }
    }

    pub fn command(&self, command: MessageCommand, ack: bool) {
        self.sender
            .send(MessagePacket { message: Message::Command(command), ack })
            .expect(CHANNEL_CLOSED_MESSAGE);
    }
    
    pub fn config(&self, command: ConfigCommand, ack: bool) {
        self.sender
            .send(MessagePacket { message: Message::Config(command), ack })
            .expect(CHANNEL_CLOSED_MESSAGE);
    }

    pub fn status(&self, command: StatusCommand) {
        self.sender
            .send(MessagePacket { message: Message::Status(command), ack: true })
            .expect(CHANNEL_CLOSED_MESSAGE);
    }

    pub fn launch(&self, objective: SearchObjective, apply: bool, print: bool) {
        self.sender
            .send(MessagePacket { message: Message::Launch { objective, apply, print }, ack: false })
            .expect(CHANNEL_CLOSED_MESSAGE);
    }
}
