mod stdio_utils;

use mintaka::config::Config;
use mintaka::game_agent::GameAgent;
use mintaka::protocol::command::Command;
use mintaka::protocol::response::Response;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::rule::RuleKind;
use std::sync::mpsc;
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
    let mut runtime_commander: Option<mpsc::Sender<Command>> = None;

    loop {
        let args = stdio_utils::read_line();

        let parameters = &args[1..];

        // https://plastovicka.github.io/protocl2en.htm
        // https://github.com/accreator/Yixin-protocol/blob/master/protocol.pdf
        let piskvork_response = match &runtime_commander {
            Some(command_sender) => {
                match args[0].as_str() {
                    "YXSTOP" => {
                        command_sender.send(Command::Abort).unwrap();
                        runtime_commander = None;
                        PiskvorkResponse::None
                    },
                    "QUIT" => {
                        std::process::exit(0);
                    },
                    _ => {
                        PiskvorkResponse::Error("command not supported.".to_string())
                    }
                }
            }
            None => {
                match args[0].as_str() {
                    "START" => {
                        let size: usize = parameters[0].parse().map_err(|_| "size parsing failed.")?;
                        if size == pos::U_BOARD_WIDTH {
                            PiskvorkResponse::Ok
                        } else {
                            PiskvorkResponse::Error("unsupported size or other error".to_string())
                        }
                    },
                    "RECTSTART" => {
                        PiskvorkResponse::Error("rectangular board is not supported or other error".to_string())
                    }
                    "BEGIN" => {
                        let (runtime_sender, runtime_receiver) = mpsc::channel();
                        let channel = game_agent.launch(runtime_receiver);
                        runtime_commander = Some(runtime_sender);

                        spawn_response_subscriber(channel);

                        PiskvorkResponse::None
                    },
                    "TURN" => {
                        let pos = parse_pos(
                            parameters.get(0).ok_or("missing row token.")?,
                            parameters.get(1).ok_or("missing column token.")?
                        )?;

                        game_agent.play(pos);

                        let (runtime_sender, runtime_receiver) = mpsc::channel();
                        let channel = game_agent.launch(runtime_receiver);
                        runtime_commander = Some(runtime_sender);

                        spawn_response_subscriber(channel);

                        PiskvorkResponse::None
                    },
                    "TAKEBACK" => {
                        let pos = parse_pos(
                            parameters.get(0).ok_or("missing row token.")?,
                            parameters.get(1).ok_or("missing column token.")?
                        )?;

                        if game_agent.recent_pos() != Some(pos) {
                            PiskvorkResponse::Error("invalid take-back position.".to_string())
                        } else {
                            game_agent.undo();

                            PiskvorkResponse::Ok
                        }
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
                                "1" | "3" => game_agent.own_color,
                                "2" => !game_agent.own_color,
                                &_ => {
                                    return Err("unknown color token.");
                                }
                            };

                            match color {
                                Color::Black => {
                                    black_moves.push(pos);
                                },
                                Color::White => {
                                    white_moves.push(pos);
                                }
                            }
                        }

                        game_agent.batch_set(
                            black_moves.into_boxed_slice(),
                            white_moves.into_boxed_slice(),
                            game_agent.own_color
                        );

                        let (runtime_sender, runtime_receiver) = mpsc::channel();
                        let channel = game_agent.launch(runtime_receiver);
                        runtime_commander = Some(runtime_sender);

                        spawn_response_subscriber(channel);

                        PiskvorkResponse::None
                    },
                    "END" => {
                        PiskvorkResponse::None
                    },
                    "INFO" => {
                        match parameters.get(0).ok_or("missing info key.")?.as_str() {
                            "timeout_match" | "time_left" => {
                                game_agent.set_remaining_time(parse_time(parameters)?);

                                PiskvorkResponse::Ok
                            },
                            "timeout_turn" => {
                                game_agent.set_turn_time(parse_time(parameters)?);

                                PiskvorkResponse::Ok
                            },
                            "max_memory" => {
                                let max_memory_in_bytes: usize =
                                    parameters.get(0).expect("missing info value.").parse().unwrap();

                                game_agent.resize_tt(max_memory_in_bytes / 1024);

                                PiskvorkResponse::Ok
                            },
                            "game_type" => {
                                match parameters.get(1)
                                    .expect("missing info value.")
                                    .chars().next().unwrap()
                                {
                                    '0' ..= '3' => PiskvorkResponse::Ok,
                                    _ => PiskvorkResponse::Error("unknown game type.".to_string()),
                                }
                            },
                            "rule" => {
                                match parameters.get(1)
                                    .expect("missing info value.")
                                    .parse::<usize>().unwrap()
                                    .count_ones()
                                {
                                    1 => {
                                        game_agent.config.rule_kind = RuleKind::FiveInARow;
                                        PiskvorkResponse::Ok
                                    },
                                    4 => {
                                        game_agent.config.rule_kind = RuleKind::Renju;
                                        PiskvorkResponse::Ok
                                    }
                                    _ => PiskvorkResponse::Error("unsupported rule.".to_string()),
                                }
                            },
                            "evaluate" => {
                                PiskvorkResponse::Ok
                            },
                            "folder" => {
                                PiskvorkResponse::Ok
                            },
                            &_ => {
                                PiskvorkResponse::Error("unsupported info key.".to_string())
                            }
                        }
                    },
                    "YXHASHCLEAR" => {
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
                    &_ => PiskvorkResponse::Error("unknown command.".to_string())
                }
            }
        };

        stdio_out(piskvork_response);
    }
}

fn spawn_response_subscriber(response_channel: mpsc::Receiver<Response>) {
    std::thread::scope(|s| {
        s.spawn(move || {
            for response in response_channel {
                let piskvork_response = match response {
                    Response::Info(message) => PiskvorkResponse::Info(message),
                    Response::Warning(message) => PiskvorkResponse::Info(format!("WARNING {}", message)),
                    Response::Error(message) => PiskvorkResponse::Error(message),
                    Response::Status { nps, total_nodes_in_1k, hash_usage, best_moves, } => {
                        PiskvorkResponse::Info(format!(
                            "statue nps={nps}, \
                            total_nodes={total_nodes_in_1k}K, \
                            hash_usage={hash_usage}, \
                            best_moves={best_moves:?}"
                        ))
                    },
                    Response::Pv(pos, pv) => {
                        PiskvorkResponse::Info(format!(
                            "pv: root={pos}, pv={pv:?}"
                        ))
                    },
                    Response::BestMove(pos, _) => {
                        PiskvorkResponse::Pos(pos)
                    },
                };

                stdio_out(piskvork_response);
            }
        });
    });
}

fn parse_pos(row: &str, col: &str) -> Result<Pos, &'static str> {
    Ok(Pos::from_cartesian(
        row.parse::<u8>().map_err(|e| "row parsing failed.")?,
        col.parse::<u8>().map_err(|e| "column parsing failed.")?,
    ))
}

fn parse_time(parameters: &[String]) -> Result<Duration, &'static str> {
    parameters.get(1)
        .ok_or("missing info value.")
        .and_then(|token| token
            .parse::<u64>()
            .map_err(|_| "time parsing failed.")
        )
        .map(Duration::from_millis)
}
