use crate::piskvork_game_manager::PiskvorkGameManager;
use mintaka::protocol::command::Command;
use mintaka::protocol::game_manager::GameManager;
use rusty_renju::notation::pos;

mod piskvork_game_manager;

enum PBrainCommand {
    Command(Command),
    Info(&'static str),
    Error(&'static str),
    Ok,
    None
}

fn main() {
    let manager = PiskvorkGameManager {};

    loop {
        let arg = "";

        // https://plastovicka.github.io/protocl2en.htm
        let pbrain_command = match arg {
            "START" => {
                let size = 15;
                if size == pos::U_BOARD_WIDTH {
                    PBrainCommand::Ok
                } else {
                    PBrainCommand::Error("ERROR unsupported size or other error")
                }
            },
            "RECTSTART" => {
                PBrainCommand::Error("ERROR rectangular board is not supported or other error")
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
            &_ => PBrainCommand::Info("ERROR unknown command.")
        };

        match pbrain_command {
            PBrainCommand::Command(command) => {
                manager.command(command);
            },
            PBrainCommand::Info(message) => {
                println!("INFO {}", message);
            },
            PBrainCommand::Error(message) => {
                println!("ERROR {}", message);
            },
            PBrainCommand::Ok => {
                println!("OK");
            },
            PBrainCommand::None => {},
        };
    }
}
