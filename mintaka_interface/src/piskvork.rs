use mintaka::config::{Config, SearchObjective};
use mintaka::game_agent::{ComputingResource, GameAgent, GameError};
use mintaka::protocol::command::Command;
use mintaka::protocol::response::{CallBackResponseSender, Response};
use mintaka::utils::thread::StdThreadProvider;
use mintaka_interface::message::{Message, MessageSender};
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::utils::byte_size::ByteSize;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

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

fn main() -> Result<(), impl Error> {
    let launched = Arc::new(AtomicBool::new(false));
    let aborted = Arc::new(AtomicBool::new(false));

    let mut game_agent = GameAgent::new(Config::default());

    let (message_sender, message_receiver) = {
        let (tx, rx) = mpsc::channel();
        (MessageSender::new(tx), rx)
    };

    spawn_command_listener(launched.clone(), aborted.clone(), message_sender.clone());

    for message in message_receiver {
        match message {
            Message::Command(command) => {
                let result = game_agent.command(command)?;
                message_sender.result(result);
            },
            Message::Launch { objective, apply, .. } => {
                launched.store(true, Ordering::Relaxed);

                let best_move = game_agent.launch::<StdThreadProvider, Instant>(
                    objective,
                    CallBackResponseSender::new(response_receiver),
                    aborted.clone()
                );

                if apply {
                    let result = game_agent.command(Command::Play(best_move.pos))?;
                    message_sender.result(result);
                }

                PiskvorkResponse::Pos(best_move.pos.unwrap());
            },
            _ => unreachable!()
        }
    }

    Ok::<(), GameError>(())
}

fn spawn_command_listener(launched: Arc<AtomicBool>, aborted: Arc<AtomicBool>, message_sender: MessageSender) {
    std::thread::spawn(move || {
        let mut buf = String::new();

        loop {
            buf.clear();
            std::io::stdin().read_line(&mut buf).expect("failed to read line");
            let args = buf.trim().split(' ').collect::<Vec<&str>>();

            if args.is_empty() {
                continue;
            }

            let piskvork_response = match_command(&launched, &aborted, &message_sender, args)
                .unwrap_or_else(|message| PiskvorkResponse::Error(message.to_string()));

            stdio_out(piskvork_response);
        }
    });
}

fn response_receiver(response: Response) {
    let piskvork_response = match response {
        Response::Begins(ComputingResource { workers, time, nodes_in_1k, tt_size }) =>
            PiskvorkResponse::Info(format!(
                "MESSAGE begins workers={workers}, running-time={time:?}, nodes={nodes_in_1k:?}k, tt-size={tt_size}"
            )),
        Response::Status { best_move, score, pv, total_nodes_in_1k, depth } =>
            PiskvorkResponse::Info(format!(
                "DEBUG status score={score}, \
                best-move={best_move:?}, \
                depth={depth}, \
                total_nodes_in_1k={total_nodes_in_1k}, \
                pv={pv:?}"
            )),
        Response::Finished => return
    };

    stdio_out(piskvork_response);
}

// https://plastovicka.github.io/protocl2en.htm
// https://github.com/accreator/Yixin-protocol/blob/master/protocol.pdf
fn match_command(
    launched: &Arc<AtomicBool>, aborted: &Arc<AtomicBool>, message_sender: &MessageSender, args: Vec<&str>
) -> Result<PiskvorkResponse, &'static str> {
    let response = if launched.load(Ordering::Relaxed) {
        match args[0] {
            "YXSTOP" => {
                aborted.store(true, Ordering::Relaxed);

                PiskvorkResponse::None
            },
            _ => return Err("command not supported."),
        }
    } else {
        match args[0] {
            "START" => {
                let size: usize = args.get(1)
                    .ok_or("missing size token.")?
                    .parse()
                    .map_err(|_| "size parsing failed.")?;

                if size == pos::U_BOARD_WIDTH {
                    PiskvorkResponse::Ok
                } else {
                    return Err("unsupported size")
                }
            },
            "RECTSTART" => return Err("rectangular board is not supported"),
            "TURN" => {
                let pos = parse_pos(
                    args.get(1).ok_or("missing row token.")?,
                    args.get(2).ok_or("missing column token.")?
                )?;

                message_sender.command(Command::Play(pos.into()));
                message_sender.launch(SearchObjective::Best, true, false);

                PiskvorkResponse::None
            },
            "BEGIN" => {
                message_sender.command(Command::Clear);
                message_sender.launch(SearchObjective::Best, true, false);

                PiskvorkResponse::None
            },
            "RESTART" => {
                message_sender.command(Command::Clear);
                PiskvorkResponse::Ok
            },
            "TAKEBACK" => {
                let pos = parse_pos(
                    args.get(1).ok_or("missing row token.")?,
                    args.get(2).ok_or("missing column token.")?
                )?;

                message_sender.command(Command::Unset { pos, color: Color::Black });

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

                message_sender.command(Command::BatchSet { player_moves, opponent_moves });

                if args[0] == "BOARD" {
                    message_sender.launch(SearchObjective::Best, false, false);
                }

                PiskvorkResponse::None
            },
            "END" => {
                PiskvorkResponse::None
            },
            "INFO" => {
                match *args.get(1).ok_or("missing info key.")? {
                    "timeout_match" | "time_left" => {
                        message_sender.command(Command::TotalTime(parse_time(&args)?));

                        PiskvorkResponse::Ok
                    },
                    "timeout_turn" => {
                        message_sender.command(Command::TurnTime(parse_time(&args)?));

                        PiskvorkResponse::Ok
                    },
                    "max_memory" => {
                        let max_memory_in_bytes: usize = args.get(1)
                            .ok_or("missing info value.")?
                            .parse()
                            .map_err(|_| "memory parsing failed.")?;

                        message_sender.command(
                            Command::MaxMemory(ByteSize::from_bytes(max_memory_in_bytes))
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
                        let rule_kind = match args.get(1)
                            .ok_or("missing info value.")?
                            .parse::<usize>()
                            .map_err(|_| "rule parsing failed.")?
                            .count_ones()
                        {
                            // 1 => Ok(RuleKind::Gomoku), // freestyle
                            // 6 => Ok(RuleKind::Gomoku), // swap2
                            4 => RuleKind::Renju,
                            _ => return Err("unsupported rule."),
                        };

                        message_sender.command(Command::Rule(rule_kind));

                        PiskvorkResponse::Ok
                    },
                    "folder" => {
                        PiskvorkResponse::Ok
                    },
                    &_ => return Err("unsupported info key."),
                }
            },
            "YXHASHCLEAR" => {
                message_sender.command(Command::Load(Box::default()));

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
        row.parse::<u8>().map_err(|_| "invalid row range.")?,
        col.parse::<u8>().map_err(|_| "invalid col range.")?,
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
