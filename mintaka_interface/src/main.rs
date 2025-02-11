use crate::pbrain_game_manager::PBrainGameManager;
use mintaka::protocol::command::Command;
use mintaka::protocol::game_manager::GameManager;
use rusty_renju::notation::pos;

mod pbrain_game_manager;
mod commandline_game_manager;

enum PBrainCommand {
    Command(Command),
    Info(&'static str),
    Error(&'static str),
    None
}

fn main() {
    let manager = PBrainGameManager {};

    loop {
        let arg = "";

        // https://plastovicka.github.io/protocl2en.htm
        let maybe_command = match arg {
            "START" => {
                let size = 15;
                if size == pos::U_BOARD_WIDTH {
                    PBrainCommand::None
                } else {
                    PBrainCommand::Error("ERROR message - unsupported size or other error")
                }
            },
            "RECTSTART" => {
                PBrainCommand::Error("ERROR message - rectangular board is not supported or other error")
            }
            "BEGIN" => {
                PBrainCommand::None
            },
            "TURN" => {
                PBrainCommand::None
            },
            "TAKEBACK" => {
                PBrainCommand::None
            },
            "BOARD" => {
                const DONE_TOKEN : &str = "DONE";

                let mut board = vec![];
                loop {
                    let line = "";
                    if line == DONE_TOKEN {
                        break;
                    }
                    board.push(line);
                }

                PBrainCommand::None
            },
            "END" => {
                PBrainCommand::None
            },
            "INFO" => {
                PBrainCommand::None
            },
            "YXHASHCLEAR" => {
                PBrainCommand::None
            },
            "YXSHOWFORBID" => {
                PBrainCommand::None
            },
            "YXSHOWINFO" => {
                PBrainCommand::None
            },
            "ABOUT" =>
                PBrainCommand::Info("name=\"mintaka\", author=\"JeongHyeon Choi\", version=\"0.1\", country=\"UNK\""),
            &_ => PBrainCommand::Info("Unknown command.")
        };

        match maybe_command {
            PBrainCommand::Command(command) => {},
            PBrainCommand::Info(_) => {}
            PBrainCommand::Error(_) => {}
            PBrainCommand::None => {}
        }
    }
}
