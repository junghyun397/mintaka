use mintaka::config::{Config, SearchObjective};
use mintaka::game_agent::{ComputingResource, GameAgent, GameError};
use mintaka::protocol::command::{Command, CompactGameState};
use mintaka::protocol::response::{CallBackResponseSender, Response};
use mintaka_interface::message::{Message, MessageCommand, MessageSender};
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::utils::byte_size::ByteSize;
use std::error::Error;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

enum PiskvorkResponse {
    Message(String),
    Debug(String),
    Error(String),
    Unknown(String),
    About(String),
    Pos(Pos),
    Ok,
    None,
}

fn stdio_out(piskvork_response: PiskvorkResponse) {
    match piskvork_response {
        PiskvorkResponse::Message(message) => {
            println!("MESSAGE {}", message);
        }
        PiskvorkResponse::Debug(message) => {
            println!("DEBUG {}", message);
        }
        PiskvorkResponse::Error(message) => {
            println!("ERROR {}", message);
        }
        PiskvorkResponse::Unknown(message) => {
            println!("UNKNOWN {}", message);
        }
        PiskvorkResponse::About(message) => {
            println!("{}", message);
        }
        PiskvorkResponse::Ok => {
            println!("OK");
        }
        PiskvorkResponse::Pos(pos) => {
            println!("{},{}", pos.col(), pos.row());
        }
        PiskvorkResponse::None => {}
    };

    std::io::stdout().flush().expect("failed to flush stdout");
}

fn main() -> Result<(), impl Error> {
    let aborted = Arc::new(AtomicBool::new(false));

    let mut game_agent = GameAgent::new(Config::default());

    let (message_sender, message_receiver) = {
        let (tx, rx) = mpsc::channel();
        (MessageSender::new(tx), rx)
    };

    spawn_command_listener(aborted.clone(), message_sender);

    let mut command_failed = false;

    for message in message_receiver {
        match message {
            Message::Command(command) => {
                let command = command.into_command(game_agent.state.board.hash_key);

                command_failed = match game_agent.command(command) {
                    Ok(_) => false,
                    Err(err) => {
                        stdio_out(PiskvorkResponse::Error(err.to_string()));
                        true
                    }
                };
            }
            Message::Launch {
                objective, apply, ..
            } => {
                if command_failed {
                    command_failed = false;
                    continue;
                }

                let best_move = game_agent.launch::<Instant>(
                    objective,
                    CallBackResponseSender::new(response_receiver),
                    Arc::new(AtomicU32::new(0)),
                    aborted.clone(),
                );

                if apply {
                    if let Err(err) = game_agent.command(Command::Play {
                        hash: game_agent.state.board.hash_key,
                        pos: best_move.best_move,
                    }) {
                        stdio_out(PiskvorkResponse::Error(err.to_string()));
                        continue;
                    }
                }

                stdio_out(PiskvorkResponse::Pos(best_move.best_move.unwrap()));
            }
            Message::Status(_) => unreachable!(),
        }
    }

    Ok::<(), GameError>(())
}

fn spawn_command_listener(aborted: Arc<AtomicBool>, message_sender: MessageSender) {
    std::thread::spawn(move || {
        let mut buf = String::new();

        loop {
            buf.clear();
            if std::io::stdin()
                .read_line(&mut buf)
                .expect("failed to read line")
                == 0
            {
                break;
            }

            let args = buf.trim().split_whitespace().collect::<Vec<&str>>();

            if args.is_empty() {
                continue;
            }

            let piskvork_response = match_command(&aborted, &message_sender, args)
                .unwrap_or_else(|message| PiskvorkResponse::Error(message.to_string()));

            stdio_out(piskvork_response);
        }
    });
}

fn response_receiver(response: Response) {
    let piskvork_response = match response {
        Response::Begins(ComputingResource {
            workers,
            time,
            nodes_in_1k,
            tt_size,
        }) => PiskvorkResponse::Message(format!(
            "begins workers={workers}, running-time={time:?}, nodes={nodes_in_1k:?}k, tt-size={tt_size}"
        )),
        Response::Status {
            best_move,
            score,
            pv,
            total_nodes_in_1k,
            selective_depth,
            ..
        } => PiskvorkResponse::Debug(format!(
            "status score={score}, \
                best-move={best_move:?}, \
                depth={selective_depth}, \
                total_nodes_in_1k={total_nodes_in_1k}, \
                pv={pv:?}"
        )),
    };

    stdio_out(piskvork_response);
}

// https://plastovicka.github.io/protocl2en.htm
// https://github.com/accreator/Yixin-protocol/blob/master/protocol.pdf
fn match_command(
    aborted: &Arc<AtomicBool>,
    message_sender: &MessageSender,
    args: Vec<&str>,
) -> Result<PiskvorkResponse, &'static str> {
    let response = match args[0] {
        "YXSTOP" => {
            aborted.store(true, Ordering::Relaxed);

            PiskvorkResponse::None
        }
        "END" => {
            std::process::exit(0);
        }
        "START" => {
            let size: usize = args
                .get(1)
                .ok_or("missing size token.")?
                .parse()
                .map_err(|_| "size parsing failed.")?;

            if size == pos::U_BOARD_WIDTH {
                message_sender.command(MessageCommand::Raw(Command::Clear));

                PiskvorkResponse::Ok
            } else {
                return Err("unsupported size");
            }
        }
        "RECTSTART" => return Err("rectangular board is not supported"),
        "TURN" => {
            let pos = parse_command_pos(&args)?;

            message_sender.command(MessageCommand::Play { pos: pos.into() });
            message_sender.launch(SearchObjective::Best, true, false);

            PiskvorkResponse::None
        }
        "BEGIN" => {
            message_sender.launch(SearchObjective::Best, true, false);

            PiskvorkResponse::None
        }
        "RESTART" => {
            message_sender.command(MessageCommand::Raw(Command::Clear));
            PiskvorkResponse::Ok
        }
        "TAKEBACK" => {
            parse_command_pos(&args)?;

            message_sender.command(MessageCommand::Undo);

            PiskvorkResponse::Ok
        }
        "BOARD" | "YXBOARD" => {
            const DONE_TOKEN: &str = "DONE";

            let mut player_moves = vec![];
            let mut opponent_moves = vec![];

            let mut buf = String::new();
            loop {
                buf.clear();
                std::io::stdin()
                    .read_line(&mut buf)
                    .map_err(|_| "failed to stdio")?;

                if buf.trim() == DONE_TOKEN {
                    break;
                }

                let [x, y, color]: [&str; 3] = buf
                    .trim()
                    .split(',')
                    .collect::<Vec<&str>>()
                    .try_into()
                    .map_err(|_| "token parsing failed.")?;

                let pos = parse_pos(x, y)?;

                match color {
                    "1" => player_moves.push(pos),
                    "2" => { opponent_moves.push(pos); }
                    &_ => return Err("unknown color token."),
                }
            }

            let (black_moves, white_moves) = if player_moves.len() >= opponent_moves.len() {
                (player_moves, opponent_moves)
            } else {
                (opponent_moves, player_moves)
            };

            if black_moves.len() > white_moves.len() + 1 {
                return Err("invalid board stones.");
            }

            let mut moves = vec![];
            for index in 0..black_moves.len() {
                moves.push(black_moves[index].into());

                if let Some(&pos) = white_moves.get(index) {
                    moves.push(pos.into());
                }
            }

            let history = History::from(moves.as_slice());
            let board: Board = (&history).into();

            message_sender.command(MessageCommand::Raw(Command::Sync(Box::new(
                CompactGameState { board, history },
            ))));

            if args[0] == "BOARD" {
                message_sender.launch(SearchObjective::Best, true, false);
            }

            PiskvorkResponse::None
        }
        "INFO" => {
            match args.get(1).copied() {
                Some("timeout_match") | Some("time_left") => {
                    if let Ok(time) = parse_time(&args) {
                        message_sender.command(MessageCommand::Raw(Command::TotalTime(time)));
                    }
                }
                Some("timeout_turn") => {
                    if let Ok(time) = parse_time(&args) {
                        message_sender.command(MessageCommand::Raw(Command::TurnTime(time)));
                    }
                }
                Some("max_memory") => {
                    if let Some(max_memory_in_bytes) =
                        args.get(2).and_then(|value| value.parse::<u64>().ok())
                    {
                        if max_memory_in_bytes > 10 * 1024 * 1024 {
                            message_sender.command(MessageCommand::Raw(Command::MaxMemory(
                                ByteSize::from_bytes(max_memory_in_bytes),
                            )));
                        }
                    }
                }
                Some("game_type") => {
                    let _ = args.get(2);
                }
                Some("rule") => {
                    if let Some(rule) = args.get(2).and_then(|value| value.parse::<usize>().ok()) {
                        let rule_kind = if rule & 4 == 4 {
                            RuleKind::Renju
                        } else {
                            RuleKind::Gomoku
                        };

                        message_sender.command(MessageCommand::Raw(Command::Rule(rule_kind)));
                    }
                }
                _ => {}
            }

            PiskvorkResponse::None
        }
        "YXHASHCLEAR" => {
            message_sender.command(MessageCommand::Raw(Command::Clear));

            PiskvorkResponse::None
        }
        "YXSHOWFORBID" => PiskvorkResponse::None,
        "YXSHOWINFO" => PiskvorkResponse::None,
        "ABOUT" => PiskvorkResponse::About(format!(
            "name=\"mintaka\", author=\"JeongHyeon Choi\", version=\"{}\", country=\"KOR\"",
            mintaka::VERSION
        )),
        &_ => PiskvorkResponse::Unknown("unknown command.".to_string()),
    };

    Ok(response)
}

fn parse_command_pos(args: &Vec<&str>) -> Result<Pos, &'static str> {
    let x = args.get(1).ok_or("missing coordinate token.")?;

    match args.get(2) {
        Some(y) if !x.contains(',') => parse_pos(x, y),
        _ => parse_pos_token(x),
    }
}

fn parse_pos_token(token: &str) -> Result<Pos, &'static str> {
    let [x, y]: [&str; 2] = token
        .split(',')
        .collect::<Vec<&str>>()
        .try_into()
        .map_err(|_| "coordinate parsing failed.")?;

    parse_pos(x, y)
}

fn parse_pos(x: &str, y: &str) -> Result<Pos, &'static str> {
    let x = x.parse::<u8>().map_err(|_| "invalid x range.")?;
    let y = y.parse::<u8>().map_err(|_| "invalid y range.")?;

    if x < pos::BOARD_WIDTH && y < pos::BOARD_WIDTH {
        Ok(Pos::from_cartesian(y, x))
    } else {
        Err("position out of range.")
    }
}

fn parse_time(parameters: &Vec<&str>) -> Result<Duration, &'static str> {
    parameters
        .get(2)
        .ok_or("missing info value.")
        .and_then(|token| token.parse::<u64>().map_err(|_| "time parsing failed."))
        .map(Duration::from_millis)
}
