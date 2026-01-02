use mintaka::config::SearchObjective;
use mintaka::protocol::command::Command;
use mintaka::protocol::results::{CommandResult, GameResult};
use std::sync::mpsc;

pub const CHANNEL_CLOSED_MESSAGE: &str = "sender channel closed.";

pub enum Message {
    Ok,
    Command(Command),
    Status(StatusCommand),
    Finished(GameResult),
    Launch {
        objective: SearchObjective,
        apply: bool,
        interactive: bool,
    },
}

pub enum StatusCommand {
    Version,
    Board { show_last_moves: bool },
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

    pub fn command(&self, command: Command) {
        self.sender.send(Message::Command(command)).expect(CHANNEL_CLOSED_MESSAGE);
    }

    pub fn status(&self, command: StatusCommand) {
        self.sender.send(Message::Status(command)).expect(CHANNEL_CLOSED_MESSAGE);
    }

    pub fn launch(&self, objective: SearchObjective, apply: bool, interactive: bool) {
        self.sender.send(Message::Launch { objective, apply, interactive }).expect(CHANNEL_CLOSED_MESSAGE);
    }

    pub fn result(&self, command_result: CommandResult) {
        match command_result.result {
            Some(game_result) =>
                self.sender.send(Message::Finished(game_result)).expect(CHANNEL_CLOSED_MESSAGE),
            None =>
                self.sender.send(Message::Ok).expect(CHANNEL_CLOSED_MESSAGE),
        }
    }

}
