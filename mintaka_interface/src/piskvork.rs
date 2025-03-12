use mintaka::config::Config;
use mintaka::game_agent::GameAgent;
use mintaka::protocol::command::Command;
use mintaka::protocol::response::Response;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::rule::RuleKind;
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
            println!("{} {}", pos.row() + 1, pos.col() + 1);
        },
        PiskvorkResponse::None => {},
    };
}

fn main() -> Result<(), &'static str> {
    let mut agent = GameAgent::new(Config::default());

    loop {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).map_err(|_| "failed to stdio")?;
        buf.make_ascii_uppercase();
        let args = buf.trim().split(' ').collect::<Vec<&str>>();

        if args.len() == 0 {
            continue;
        }

        let command = args[0];
        let parameters = &args[1..];

        // https://plastovicka.github.io/protocl2en.htm
        // https://github.com/accreator/Yixin-protocol/blob/master/protocol.pdf
        let piskvork_response = match command {
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
                let channel = agent.launch();
                spawn_response_subscriber(channel);

                PiskvorkResponse::None
            },
            "TURN" => {
                let pos = parse_pos(
                    parameters.get(0).ok_or("missing row token.")?,
                    parameters.get(1).ok_or("missing column token.")?
                )?;

                agent.set(pos, !agent.own_color);

                let channel = agent.launch();
                spawn_response_subscriber(channel);

                PiskvorkResponse::None
            },
            "TAKEBACK" => {
                let pos = parse_pos(
                    parameters.get(0).ok_or("missing row token.")?,
                    parameters.get(1).ok_or("missing column token.")?
                )?;

                agent.unset(pos, !agent.own_color);

                PiskvorkResponse::Ok
            },
            "BOARD" | "YXBOARD" => {
                const DONE_TOKEN: &str = "DONE";

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
                        "1" => agent.own_color,
                        "2" => !agent.own_color,
                        "3" => agent.own_color,
                        &_ => {
                            return Err("unknown color token.");
                        }
                    };

                    agent.set(pos, color);
                }

                let channel = agent.launch();
                spawn_response_subscriber(channel);

                PiskvorkResponse::None
            },
            "END" => {
                PiskvorkResponse::None
            },
            "INFO" => {
                match *args.get(0).ok_or("missing info key.")? {
                    "timeout_match" | "time_left" => {
                        agent.time_manager.total_remaining = parse_time(parameters)?;

                        PiskvorkResponse::Ok
                    },
                    "timeout_turn" => {
                        agent.time_manager.overhead = parse_time(parameters)?;

                        PiskvorkResponse::Ok
                    },
                    "max_memory" => {
                        let max_memory_in_bytes: usize =
                            parameters.get(0).expect("missing info value.").parse().unwrap();

                        agent.resize_tt(max_memory_in_bytes / 1024);

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
                                agent.config.rule_kind = RuleKind::FiveInARow;
                                PiskvorkResponse::Ok
                            },
                            4 => {
                                agent.config.rule_kind = RuleKind::Renju;
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
            "YXSTOP" => {
                agent.command(Command::Abort);

                PiskvorkResponse::None
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
        };

        stdio_out(piskvork_response);
    }
}

fn spawn_response_subscriber(response_channel: std::sync::mpsc::Receiver<Response>) {
    std::thread::spawn(move || {
        for response in response_channel {
            let piskvork_response = match response {
                Response::Info(message) => PiskvorkResponse::Info(message),
                Response::Warning(message) => PiskvorkResponse::Info(message),
                Response::Error(message) => PiskvorkResponse::Error(message),
                Response::Status(status) => {
                    PiskvorkResponse::Info("status".to_string())
                },
                Response::Pv(pos, pv) => {
                    PiskvorkResponse::Info("pv".to_string())
                },
                Response::BestMove(pos, _) => {
                    PiskvorkResponse::Pos(pos)
                },
            };

            stdio_out(piskvork_response);
        }
    });
}

fn parse_pos(row: &str, col: &str) -> Result<Pos, &'static str> {
    Ok(Pos::from_cartesian(
        row.parse::<u8>().map_err(|e| "row parsing failed.")? - 1,
        col.parse::<u8>().map_err(|e| "column parsing failed.")? - 1,
    ))
}

fn parse_time(parameters: &[&str]) -> Result<Duration, &'static str> {
    parameters.get(0)
        .ok_or("missing info value.")
        .and_then(|token| token
            .parse::<u64>()
            .map_err(|_| "time parsing failed.")
        )
        .map(Duration::from_millis)
}
