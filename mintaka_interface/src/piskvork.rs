mod stdio_utils;

use mintaka::config::Config;
use mintaka::game_agent::GameAgent;
use mintaka::protocol::command::Command;
use mintaka::protocol::message::{CommandSender, Message, ResponseSender};
use mintaka::protocol::response::Response;
use rusty_renju::board::Board;
use rusty_renju::history::{Action, History};
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::rule::RuleKind;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

enum PiskvorkResponse {
    Info(String),
    Error(String),
    Pos(Pos),
    Ok,
    None
}

fn stdio_out(piskvork_response: PiskvorkResponse) {
    match piskvork_response {
        PiskvorkResponse::Info(message) => {
            println!("INFO {}", message);
        },
        PiskvorkResponse::Error(message) => {
            println!("ERROR {}", message);
        },
        PiskvorkResponse::Ok => {
            println!("OK");
        },
        PiskvorkResponse::Pos(pos) => {
            println!("{},{}", pos.row() + 1, pos.col() + 1);
        },
        PiskvorkResponse::None => {},
    };
}

fn main() -> Result<(), &'static str> {
    let mut game_agent = GameAgent::new(Config::default());

    let launched = Arc::new(AtomicBool::new(false));

    let (command_sender, response_sender, message_receiver) = {
        let (message_sender, message_receiver) = mpsc::channel();
        (CommandSender::new(message_sender.clone()), ResponseSender::new(message_sender), message_receiver)
    };

    spawn_command_listener(launched.clone(), command_sender);

    for message in message_receiver {
        match message {
            Message::Command(command) => {
                game_agent.command(command)?;
            },
            Message::Response(response) => {
                let piskvork_response = match response {
                    Response::Info(message) => PiskvorkResponse::Info(message),
                    Response::Warning(message) => PiskvorkResponse::Info(message),
                    Response::Error(message) => PiskvorkResponse::Error(message),
                    Response::Status { eval, total_nodes_in_1k, hash_usage, best_moves } => {
                        PiskvorkResponse::Info(format!(
                            "status eval={eval} \
                            total-nodes={total_nodes_in_1k}K, \
                            hash-usage={hash_usage}, \
                            best-moves={best_moves:?}"
                        ))
                    },
                    Response::Pv(pvs) => {
                        PiskvorkResponse::Info(format!("pvs={pvs:?}"))
                    },
                    Response::BestMove(pos, _) => {
                        launched.store(false, Ordering::Relaxed);

                        game_agent.command(Command::Play(Action::Move(pos)))?;

                        PiskvorkResponse::Pos(pos)
                    }
                };

                stdio_out(piskvork_response);
            },
            Message::Launch => {
                launched.store(true, Ordering::Relaxed);
                game_agent.launch(response_sender.clone());
            },
            Message::Abort => {
                launched.store(false, Ordering::Relaxed);
                game_agent.abort();
            },
            Message::Quit => {
                break;
            },
            _ => unreachable!()
        }
    }

    Ok(())
}

fn spawn_command_listener(launched: Arc<AtomicBool>, command_sender: CommandSender) {
    std::thread::spawn(move || {
        let mut buf = String::new();

        loop {
            buf.clear();
            std::io::stdin().read_line(&mut buf).expect("failed to read line");
            let args = buf.trim().split(' ').collect::<Vec<&str>>();

            if args.len() == 0 {
                continue;
            }

            let piskvork_response = match_command(&launched, &command_sender, args)
                .unwrap_or_else(|message| PiskvorkResponse::Error(message.to_string()));

            stdio_out(piskvork_response);
        }
    });
}

// https://plastovicka.github.io/protocl2en.htm
// https://github.com/accreator/Yixin-protocol/blob/master/protocol.pdf
fn match_command(
    launched: &Arc<AtomicBool>, command_sender: &CommandSender, args: Vec<&str>
) -> Result<PiskvorkResponse, &'static str> {
    let response = if launched.load(Ordering::Relaxed) {
        match args[0] {
            "YXSTOP" => {
                command_sender.abort();

                PiskvorkResponse::None
            },
            "QUIT" => {
                command_sender.quit();

                PiskvorkResponse::None
            },
            _ => return Err("command not supported."),
        }
    } else {
        match args[0] {
            "START" => {
                let size: usize = args[1].parse().map_err(|_| "size parsing failed.")?;

                if size == pos::U_BOARD_WIDTH {
                    PiskvorkResponse::Ok
                } else {
                    return Err("unsupported size or other error")
                }
            },
            "RECTSTART" => return Err("rectangular board is not supported or other error"),
            "TURN" => {
                let pos = parse_pos(
                    args.get(1).ok_or("missing row token.")?,
                    args.get(2).ok_or("missing column token.")?
                )?;

                command_sender.command(Command::Play(Action::Move(pos)));
                command_sender.launch();

                PiskvorkResponse::None
            },
            "BEGIN" => {
                command_sender.launch();

                PiskvorkResponse::None
            },
            "TAKEBACK" => {
                let pos = parse_pos(
                    args.get(1).ok_or("missing row token.")?,
                    args.get(2).ok_or("missing column token.")?
                )?;

                command_sender.command(Command::Unset { pos, color: Color::Black });

                PiskvorkResponse::Ok
            },
            "BOARD" | "YXBOARD" => {
                const DONE_TOKEN: &str = "DONE";

                let mut player_moves = vec![];
                let mut opponent_moves = vec![];

                let mut buf = String::new();
                loop {
                    buf.clear();
                    std::io::stdin().read_line(&mut buf).map_err(|_| "failed to stdio")?;

                    if buf.trim() == DONE_TOKEN {
                        break;
                    }

                    let [row, col, color]: [&str; 3] = buf.trim()
                        .split(',')
                        .collect::<Vec<&str>>()
                        .try_into()
                        .map_err(|_| "token parsing failed.")?;

                    let pos = parse_pos(row, col)?;

                    match color {
                        "1" => {
                            player_moves.push(pos);
                        },
                        "2" => {
                            opponent_moves.push(pos);
                        },
                        &_ => return Err("unknown color token.")
                    };
                }

                command_sender.command(Command::BatchSet { player_moves, opponent_moves });

                if args[0] == "BOARD" {
                    command_sender.launch();
                }

                PiskvorkResponse::None
            },
            "END" => {
                command_sender.quit();

                PiskvorkResponse::None
            },
            "INFO" => {
                match *args.get(1).ok_or("missing info key.")? {
                    "timeout_match" | "time_left" => {
                        command_sender.command(Command::TotalTime(parse_time(&args)?));

                        PiskvorkResponse::Ok
                    },
                    "timeout_turn" => {
                        command_sender.command(Command::TurnTime(parse_time(&args)?));

                        PiskvorkResponse::Ok
                    },
                    "max_memory" => {
                        let max_memory_in_bytes: usize = args.get(1)
                            .ok_or("missing info value.")?
                            .parse()
                            .map_err(|_| "memory parsing failed.")?;

                        command_sender.command(
                            Command::MaxMemory { in_kib: max_memory_in_bytes / 1024 }
                        );

                        PiskvorkResponse::Ok
                    },
                    "game_type" => {
                        match args.get(1)
                            .ok_or("missing info value.")?
                            .chars()
                            .next().unwrap()
                        {
                            '0' ..= '3' => PiskvorkResponse::Ok,
                            _ => return Err("unknown game type."),
                        }
                    },
                    "rule" => {
                        let rule_kind: Result<RuleKind, &str> = match args.get(1)
                            .ok_or("missing info value.")?
                            .parse::<usize>()
                            .map_err(|_| "rule parsing failed.")?
                            .count_ones()
                        {
                            // 1 => Ok(RuleKind::Gomoku),
                            // 6 => Ok(RuleKind::Gomoku), // swap2
                            4 => Ok(RuleKind::Renju),
                            _ => return Err("unsupported rule."),
                        };

                        command_sender.command(Command::Rule(rule_kind?));

                        PiskvorkResponse::Ok
                    },
                    "folder" => {
                        PiskvorkResponse::Ok
                    },
                    &_ => return Err("unsupported info key."),
                }
            },
            "YXHASHCLEAR" => {
                command_sender.command(
                    Command::Load(Box::from(Board::default()), History::default())
                );

                PiskvorkResponse::None
            },
            "YXSHOWFORBID" => {
                PiskvorkResponse::None
            },
            "YXSHOWINFO" => {
                PiskvorkResponse::None
            },
            "ABOUT" =>
                PiskvorkResponse::Info(
                    format!(
                        "name=\"mintaka\",\
                                author=\"JeongHyeon Choi\",\
                                version=\"{}\",\
                                country=\"KOR\"",
                        mintaka::VERSION
                    ).to_string()
                ),
            &_ => return Err("unknown command.")
        }
    };

    Ok(response)
}

fn parse_pos(row: &str, col: &str) -> Result<Pos, &'static str> {
    Ok(Pos::from_cartesian(
        row.parse::<u8>().map_err(|e| "row parsing failed.")?,
        col.parse::<u8>().map_err(|e| "column parsing failed.")?,
    ))
}

fn parse_time(parameters: &Vec<&str>) -> Result<Duration, &'static str> {
    parameters.get(2)
        .ok_or("missing info value.")
        .and_then(|token| token
            .parse::<u64>()
            .map_err(|_| "time parsing failed.")
        )
        .map(Duration::from_millis)
}
