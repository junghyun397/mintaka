mod stdio_utils;

use mintaka::config::Config;
use mintaka::game_agent::{GameAgent, GameCommand};
use mintaka::protocol::command::Command;
use mintaka::protocol::response::Response;
use mintaka::protocol::runtime_command::RuntimeCommand;
use rusty_renju::history::Action;
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

fn spawn_command_listener(launched: Arc<AtomicBool>, command_sender: mpsc::Sender<GameCommand>) {
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

fn main() -> Result<(), &'static str> {
    let mut game_agent = GameAgent::new(Config::default());

    let launched = Arc::new(AtomicBool::new(false));

    let (command_sender, command_receiver) = mpsc::channel();

    let mut runtime_channel: Option<(mpsc::Sender<RuntimeCommand>, mpsc::Receiver<Response>)> = None;

    spawn_command_listener(launched.clone(), command_sender);

    for game_command in command_receiver {
        while let Some(response) = runtime_channel.as_ref()
            .and_then(|(_, receiver)| receiver.try_recv().ok())
        {
            let piskvork_response = match response {
                Response::Info(message) => PiskvorkResponse::Info(message),
                Response::Warning(message) => PiskvorkResponse::Info(message),
                Response::Error(message) => PiskvorkResponse::Error(message),
                Response::Status { total_nodes_in_1k, hash_usage, best_moves } => {
                    PiskvorkResponse::Info(format!(
                        "status total-nodes={total_nodes_in_1k}K, hash-usage={hash_usage}, best-moves={best_moves:?}"
                    ))
                },
                Response::Pv(pos, pv) => {
                    PiskvorkResponse::Info(format!("pv pos={}, pv={}", pos, pv))
                },
                Response::BestMove(pos, _) => {
                    runtime_channel = None;
                    game_agent.play(pos);
                    PiskvorkResponse::Pos(pos)
                }
            };

            stdio_out(piskvork_response);
        }

        match game_command {
            GameCommand::Command(command) => {
                game_agent.command(command);
            },
            GameCommand::RuntimeCommand(runtime_command) => {
                if let Some((runtime_commander, _)) = runtime_channel.as_ref() {
                    runtime_commander.send(runtime_command).unwrap();
                } else {
                    return Err("runtime channel is not available.");
                }
            },
            GameCommand::Launch => {
                let (runtime_command_tx, runtime_command_rx) = mpsc::channel();
                let response_rx = game_agent.launch(runtime_command_rx);

                runtime_channel = Some((runtime_command_tx, response_rx));
            },
            GameCommand::Quite => {
                break;
            }
        }
    }

    Ok(())
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

// https://plastovicka.github.io/protocl2en.htm
// https://github.com/accreator/Yixin-protocol/blob/master/protocol.pdf
fn match_command(
    launched: &Arc<AtomicBool>, command_sender: &mpsc::Sender<GameCommand>, args: Vec<&str>
) -> Result<PiskvorkResponse, &'static str> {
    if launched.load(Ordering::Relaxed) {
        match args[0] {
            "YXSTOP" => {
                command_sender.send(GameCommand::RuntimeCommand(RuntimeCommand::Abort)).unwrap();
                Ok(PiskvorkResponse::None)
            },
            "QUIT" => {
                command_sender.send(GameCommand::Quite).unwrap();
                Ok(PiskvorkResponse::None)
            },
            _ => {
                Err("command not supported.")
            }
        }
    } else {
        match args[0] {
            "START" => {
                let size: usize = args[1].parse().map_err(|_| "size parsing failed.")?;
                if size == pos::U_BOARD_WIDTH {
                    Ok(PiskvorkResponse::Ok)
                } else {
                    Err("unsupported size or other error")
                }
            },
            "RECTSTART" => {
                Err("rectangular board is not supported or other error")
            }
            "BEGIN" => {
                command_sender.send(GameCommand::Launch).unwrap();
                Ok(PiskvorkResponse::None)
            },
            "TURN" => {
                let pos = parse_pos(
                    args.get(1).ok_or("missing row token.")?,
                    args.get(2).ok_or("missing column token.")?
                )?;

                command_sender.send(GameCommand::Command(
                    Command::Play(Action::Move(pos))
                )).unwrap();

                command_sender.send(GameCommand::Launch).unwrap();

                Ok(PiskvorkResponse::None)
            },
            "TAKEBACK" => {
                let pos = parse_pos(
                    args.get(1).ok_or("missing row token.")?,
                    args.get(2).ok_or("missing column token.")?
                )?;

                command_sender.send(GameCommand::Command(
                    Command::Unset { pos, color: Color::Black }
                )).unwrap();

                Ok(PiskvorkResponse::Ok)
            },
            "BOARD" | "YXBOARD" => {
                const DONE_TOKEN: &str = "DONE";

                let mut black_moves = vec![];
                let mut white_moves = vec![];

                loop {
                    let mut buf = String::new();
                    std::io::stdin().read_line(&mut buf).map_err(|_| "failed to stdio")?;

                    if buf.trim() == DONE_TOKEN {
                        break;
                    }

                    let [row, col, color]: [&str; 3] = buf.trim()
                        .split(',')
                        .collect::<Vec<&str>>()
                        .try_into()
                        .map_err(|_| "token parsing failed.")
                        ?;

                    let pos = parse_pos(row, col)?;

                    let color = match color {
                        "1" | "3" => Ok(Color::Black),
                        "2" => Ok(!Color::Black),
                        &_ => Err("unknown color token.")
                    }?;

                    match color {
                        Color::Black => {
                            black_moves.push(pos);
                        },
                        Color::White => {
                            white_moves.push(pos);
                        }
                    }
                }

                Ok(PiskvorkResponse::None)
            },
            "END" => {
                command_sender.send(GameCommand::Quite).unwrap();
                Ok(PiskvorkResponse::None)
            },
            "INFO" => {
                match *args.get(1).ok_or("missing info key.")? {
                    "timeout_match" | "time_left" => {
                        command_sender.send(GameCommand::Command(
                            Command::TotalTime(parse_time(&args)?))
                        ).unwrap();

                        Ok(PiskvorkResponse::Ok)
                    },
                    "timeout_turn" => {
                        command_sender.send(GameCommand::Command(
                            Command::TurnTime(parse_time(&args)?)
                        )).unwrap();

                        Ok(PiskvorkResponse::Ok)
                    },
                    "max_memory" => {
                        let max_memory_in_bytes: usize =
                            args.get(1).expect("missing info value.").parse().unwrap();

                        command_sender.send(GameCommand::Command(
                            Command::MaxMemory { in_kib: max_memory_in_bytes / 1024 }
                        )).unwrap();

                        Ok(PiskvorkResponse::Ok)
                    },
                    "game_type" => {
                        match args.get(1)
                            .expect("missing info value.")
                            .chars().next().unwrap()
                        {
                            '0' ..= '3' => Ok(PiskvorkResponse::Ok),
                            _ => Err("unknown game type."),
                        }
                    },
                    "rule" => {
                        let rule_kind = match args.get(1)
                            .expect("missing info value.")
                            .parse::<usize>().unwrap()
                            .count_ones()
                        {
                            1 => Ok(RuleKind::Gomoku),
                            4 => Ok(RuleKind::Renju),
                            _ => Err("unsupported rule."),
                        }?;

                        command_sender.send(GameCommand::Command(
                            Command::Rule(rule_kind)
                        )).unwrap();

                        Ok(PiskvorkResponse::Ok)
                    },
                    "evaluate" => {
                        Ok(PiskvorkResponse::Ok)
                    },
                    "folder" => {
                        Ok(PiskvorkResponse::Ok)
                    },
                    &_ => {
                        Err("unsupported info key.")
                    }
                }
            },
            "YXHASHCLEAR" => {
                Ok(PiskvorkResponse::None)
            },
            "YXSHOWFORBID" => {
                Ok(PiskvorkResponse::None)
            },
            "YXSHOWINFO" => {
                Ok(PiskvorkResponse::None)
            },
            "ABOUT" =>
                Ok(PiskvorkResponse::Info(
                    format!(
                        "name=\"mintaka\",\
                                author=\"JeongHyeon Choi\",\
                                version=\"{}\",\
                                country=\"KOR\"",
                        mintaka::VERSION
                    ).to_string()
                )),
            &_ => Err("unknown command.")
        }
    }
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
