use mintaka::config::{Config, SearchObjective};
use mintaka::game_agent::{ComputingResource, GameAgent, GameError};
use mintaka::game_state::GameState;
use mintaka::protocol::command::Command;
use mintaka::protocol::response::{CallBackResponseSender, Response};
use mintaka::protocol::time::{TimeUnit, TimeValue};
use mintaka::protocol::timer::Timer;
use mintaka_interface::message::{ConfigCommand, Message, MessageCommand, MessagePacket, MessageSender, StatusCommand};
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::utils::byte_size::ByteSize;
use rusty_renju::utils::empty::Empty;
use std::convert::Into;
use std::error::Error;
use std::io::{BufRead, Write};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};
use mintaka::memo::transposition_table::TranspositionTable;
use mintaka::protocol::nodes::Nodes;

pub fn entry<const R: RuleKind>() -> Result<(), impl Error> {
    piskvork_protocol::<R>()
}

const PROTOCOL_MARGIN_IN_MS: u64 = 30;
const ENGINE_SYSTEM_MEMORY: ByteSize = ByteSize::from_mib(64);

enum PiskvorkError {
    Error(String),
    Unknown,
}

enum PiskvorkResponse {
    Message(String),
    Debug(String),
    Unknown(String),
    About(String),
    Pos(Pos),
    Forbid(Option<Vec<Pos>>),
    Ok,
}

fn stdio_out(piskvork_response: Result<PiskvorkResponse, PiskvorkError>) {
    match piskvork_response {
        Ok(PiskvorkResponse::Message(message)) => {
            println!("MESSAGE {}", message);
        }
        Ok(PiskvorkResponse::Debug(message)) => {
            println!("DEBUG {}", message);
        }
        Ok(PiskvorkResponse::Unknown(message)) => {
            println!("UNKNOWN {}", message);
        }
        Ok(PiskvorkResponse::About(message)) => {
            println!("{}", message);
        }
        Ok(PiskvorkResponse::Ok) => {
            println!("OK");
        }
        Ok(PiskvorkResponse::Pos(pos)) => {
            println!("{},{}", pos.col(), pos.row());
        }
        Ok(PiskvorkResponse::Forbid(positions)) => {
            match positions {
                Some(positions) => println!(
                    "FORBID {}.",
                    positions.iter()
                        .map(|pos| format!("{:02}{:02}", pos.col(), pos.row()))
                        .collect::<Vec<_>>()
                        .join("")
                ),
                None => println!("FORBID ."),
            }
        }
        Err(PiskvorkError::Error(message)) => {
            println!("ERROR {}", message);
        }
        Err(PiskvorkError::Unknown) => {
            println!("UNKNOWN");
        }
    };

    std::io::stdout().flush().expect("failed to flush stdout");
}

fn print_response(response: Response) {
    let response = match response {
        Response::Begins(ComputingResource { workers, time_unit, time_limit }) =>
            format!(
                "begins workers={workers},\
                 time-unit={time_unit}, \
                 running-budget={time_limit:?}"
            ),
        Response::Status { best_move, score, pv, total_nodes: total_nodes_in_1k, selective_depth, .. } =>
            format!(
                "status score={score}, \
                best-move={best_move:?}, \
                depth={selective_depth}, \
                total_nodes_in_1k={total_nodes_in_1k}, \
                pv={pv:?}"
            ),
    };

    stdio_out(Ok(PiskvorkResponse::Debug(response)));
}

fn piskvork_protocol<const R: RuleKind>() -> Result<(), impl Error> {
    let aborted = Arc::new(AtomicBool::new(false));

    let mut config = Presets::FASTGAME;

    let mut game_agent = GameAgent::<R>::new(config);

    let mut timer = config.initial_timer;

    let (message_sender, message_receiver) = {
        let (tx, rx) = mpsc::channel();
        (MessageSender::new(tx), rx)
    };

    spawn_command_listener::<R>(aborted.clone(), message_sender);

    for MessagePacket { message, ack } in message_receiver {
        match message {
            Message::Command(command) => {
                let command = command.into_command(&config, game_agent.state.board.hash_key);

                let result = game_agent.command(command);

                if let Err(err) = result {
                    stdio_out(Err(PiskvorkError::Error(err.to_string())));
                    continue;
                }
            }
            Message::Launch { objective, apply, .. } => {
                let best_move = game_agent.launch::<Instant>(
                    config,
                    timer,
                    objective,
                    CallBackResponseSender::new(print_response),
                    Arc::new(AtomicU32::new(0)),
                    aborted.clone(),
                );

                if apply {
                    let result = game_agent.command(Command::Play {
                        hash: game_agent.state.board.hash_key,
                        pos: best_move.best_move,
                        draw_condition: config.draw_condition,
                    });

                    match result {
                        Ok(_) => {
                            timer.consume(TimeValue::from_duration(best_move.time_elapsed))
                        }
                        Err(err) => {
                            stdio_out(Err(PiskvorkError::Error(err.to_string())));
                            continue;
                        }
                    }
                }

                stdio_out(Ok(PiskvorkResponse::Pos(best_move.best_move.unwrap_or(Pos::from_cartesian(7, 7)))));
            }
            Message::Config(ConfigCommand::TotalTime(total)) => {
                timer.total_remaining = total.map(|total| TimeValue::from_value(total, TimeUnit::Clock));
            }
            Message::Config(ConfigCommand::IncrementTime(increment)) => {
                let increment = TimeValue::from_value(increment, TimeUnit::Clock);

                config.initial_timer.increment = increment;
                timer.increment = increment;
            }
            Message::Config(ConfigCommand::TurnTime(turn)) => {
                let turn = turn.map(|turn| TimeValue::from_value(turn, TimeUnit::Clock));

                config.initial_timer.turn = turn;
                timer.turn = turn
            }
            Message::Config(ConfigCommand::Workers(workers)) => {
                config.workers = workers.unwrap_or_else(||
                    std::thread::available_parallelism().map_or_else(|_| 1, |n| n.get()) as u32
                );
            }
            Message::Config(ConfigCommand::MaxMemory(max_size)) => {
                const GENERAL_NPMS: Nodes = Nodes::from_in_1k(2);

                let optimal_size = config.initial_timer.turn
                    .or_else(|| config.initial_timer.total_remaining.map(|time| time / 20))
                    .map(|time| {
                        let nodes = match config.initial_timer.time_unit {
                            TimeUnit::Clock => GENERAL_NPMS * time.to_duration().as_millis() as u32,
                            TimeUnit::Nodes => time.to_nodes()
                        };

                        TranspositionTable::optimal_size(nodes)
                    });

                let size = match (optimal_size, max_size) {
                    (Some(optimal), Some(max)) => optimal.min(max - ENGINE_SYSTEM_MEMORY),
                    (Some(size), None) => size,
                    (None, Some(max)) => max - ENGINE_SYSTEM_MEMORY,
                    (None, None) => ByteSize::from_mib(2048)
                };

                let _ = game_agent.command(Command::RebuildTT(size));
            }
            Message::Config(ConfigCommand::ResizeTT(_)) => unreachable!(),
            Message::Config(ConfigCommand::TimeUnit(_)) => unreachable!(),
            Message::Config(ConfigCommand::MaxDepth(_)) => unreachable!(),
            Message::Status(status_command) => {
                match status_command {
                    StatusCommand::Forbid => {
                        let positions = (game_agent.state.board.player_color == Color::Black).then(||
                            game_agent.state.board.patterns.forbidden_field.iter_hot_pos().collect()
                        );

                        stdio_out(Ok(PiskvorkResponse::Forbid(positions)));
                    }
                    StatusCommand::Version => {
                        stdio_out(Ok(PiskvorkResponse::About(
                            format!(
                                "name=\"mintaka\", author=\"JeongHyeon Choi\", version=\"{}\", country=\"KOR\"",
                                mintaka::VERSION,
                            )
                        )));
                    }
                    StatusCommand::Board { .. } => unreachable!(),
                    StatusCommand::History => unreachable!(),
                    StatusCommand::Time => unreachable!(),
                }

                continue;
            }
        }

        if ack {
            stdio_out(Ok(PiskvorkResponse::Ok));
        }
    }

    Ok::<(), GameError>(())
}

// https://plastovicka.github.io/protocl2en.htm
// https://github.com/accreator/Yixin-protocol/blob/master/protocol.pdf
fn match_command<const R: RuleKind>(
    aborted: &Arc<AtomicBool>,
    message_sender: &MessageSender,
    line: String,
) -> Result<(), PiskvorkError> {
    let mut args = line.split(' ').into_iter();

    let Some(command) = args.next() else {
        return Err(PiskvorkError::Error("command not provided".to_string()))
    };

    match command {
        // basic commands
        "START" => {
            let size: usize = args
                .next()
                .ok_or(PiskvorkError::Error("missing size token.".to_string()))?
                .parse()
                .map_err(|_| PiskvorkError::Error("size parsing failed.".to_string()))?;

            if size == pos::U_BOARD_WIDTH {
                message_sender.command(MessageCommand::Command(Command::Clear), true);
            } else {
                return Err(PiskvorkError::Error("unsupported size".to_string()));
            }
        }
        "BEGIN" => {
            message_sender.launch(SearchObjective::Best, true, false);
        }
        "INFO" => {
            match args.next().map(str::to_lowercase).as_deref() {
                Some("timeout_match") | Some("time_left") => {
                    if let Ok(time) = parse_time(args.next()) {
                        message_sender.config(
                            ConfigCommand::TotalTime(
                                (time != 0).then_some(time.saturating_sub(PROTOCOL_MARGIN_IN_MS))
                            ),
                            false,
                        );
                    } else {
                        return Err(PiskvorkError::Error("invalid time value".to_string()));
                    }
                }
                Some("timeout_turn") => {
                    if let Ok(time) = parse_time(args.next()) {
                        message_sender.config(
                            ConfigCommand::TurnTime(
                                (time != 0).then_some(time.saturating_sub(PROTOCOL_MARGIN_IN_MS))
                            ),
                            false,
                        );
                    } else {
                        return Err(PiskvorkError::Error("invalid time value".to_string()));
                    }
                }
                Some("max_memory") => {
                    if let Some(max_memory_in_bytes) = args.next()
                        && let Some(max_memory_in_bytes) = max_memory_in_bytes.parse::<u64>().ok()
                        && let max_memory = (max_memory_in_bytes != 0)
                            .then_some(ByteSize::from_bytes(max_memory_in_bytes))
                        && max_memory.is_none_or(|max_memory| max_memory > ENGINE_SYSTEM_MEMORY)
                    {
                        message_sender.config(ConfigCommand::MaxMemory(max_memory), false);
                    } else {
                        return Err(PiskvorkError::Error("invalid memory value".to_string()));
                    }
                }
                Some("thread_num") => {
                    if let Some(workers) = args.next()
                        && let Some(workers) = workers.parse::<u32>().ok()
                    {
                        message_sender.config(
                            ConfigCommand::Workers((workers != 0).then_some(workers)),
                            false,
                        );
                    } else {
                        return Err(PiskvorkError::Error("invalid thread value".to_string()));
                    }
                }
                Some("game_type") => {
                    let _ = args.next();
                }
                Some("rule") => {
                    if let Some(rule) = args.next().and_then(|value| value.parse::<usize>().ok()) {
                        if match rule {
                            0 => RuleKind::Freestyle,
                            1 => RuleKind::Gomoku,
                            2 | 4 => RuleKind::Renju,
                            _ => return Err(PiskvorkError::Unknown),
                        } != R {
                            return Err(PiskvorkError::Error("unsupported rule".to_string()));
                        }
                    } else {
                        return Err(PiskvorkError::Error("invalid rule value".to_string()));
                    }
                }
                _ => return Ok(()),
            }
        }
        "BOARD" | "YXBOARD" => {
            const DONE_TOKEN: &str = "DONE";

            let mut sequence = vec![];

            let mut buf = String::new();
            loop {
                buf.clear();
                std::io::stdin()
                    .read_line(&mut buf)
                    .map_err(|_| PiskvorkError::Error("failed to stdio".to_string()))?;

                if buf.trim() == DONE_TOKEN {
                    break;
                }

                let [x, y, color]: [&str; 3] = buf
                    .trim()
                    .split(',')
                    .collect::<Vec<&str>>()
                    .try_into()
                    .map_err(|_| PiskvorkError::Error("coordinate parsing failed".to_string()))?;

                let pos = parse_pos(x, y)?;

                match color {
                    "1" => sequence.push((pos, true)),
                    "2" => sequence.push((pos, false)),
                    "3" => {},
                    &_ => return Err(PiskvorkError::Error("unknown color token".to_string()))
                }
            }

            let own_color = sequence.first()
                .map(|&(_, own)| if own { Color::Black } else { Color::White })
                .unwrap_or(Color::Black);

            let mut game_state = GameState::<R>::empty();

            for (pos, own) in sequence {
                if own != (game_state.board.player_color == own_color) {
                    game_state.pass_mut();
                }

                if !game_state.board.is_legal_move(pos) {
                    return Err(PiskvorkError::Error("illegal move".to_string()));
                }

                game_state.play_mut(pos);
            }

            message_sender.command(MessageCommand::Command(Command::Init(Box::new((&game_state).into()))), false);

            if command == "BOARD" {
                message_sender.launch(SearchObjective::Best, true, false);
            }
        }
        "TURN" => {
            let pos = parse_command_pos(&mut args)?;

            message_sender.command(MessageCommand::Play { pos: pos.into() }, false);
            message_sender.launch(SearchObjective::Best, true, false);
        }
        "END" => {
            std::process::exit(0);
        }
        "STOP" | "YXSTOP" => {
            aborted.store(true, Ordering::Relaxed);
        }
        // extended commands
        "RECTSTART" => return Err(PiskvorkError::Error("rectangular board is not supported".to_string())),
        "RESTART" => {
            message_sender.command(MessageCommand::Command(Command::Clear), true);
        }
        "TAKEBACK" => {
            parse_command_pos(&mut args)?;

            message_sender.command(MessageCommand::Undo, true);
        }
        "ABOUT" => {
            message_sender.status(StatusCommand::Version);
        },
        "YXSHOWFORBID" => {
            message_sender.status(StatusCommand::Forbid);
        }
        &_ => return Err(PiskvorkError::Unknown),
    }

    Ok(())
}

fn spawn_command_listener<const R: RuleKind>(
    aborted: Arc<AtomicBool>,
    message_sender: MessageSender,
) {
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let stdin_lines = stdin.lock().lines();

        for line in stdin_lines.map(Result::unwrap) {
            let result = match_command::<R>(&aborted, &message_sender, line.to_uppercase());

            if let Err(error) = result {
                stdio_out(Err(error));
            }
        }
    });
}

fn parse_command_pos<'a>(args: &mut impl Iterator<Item = &'a str>) -> Result<Pos, PiskvorkError> {
    let x = args.next().ok_or(PiskvorkError::Error("missing coordinate token.".to_string()))?;

    match args.next() {
        Some(y) if !x.contains(',') => parse_pos(x, y),
        _ => parse_pos_token(x),
    }
}

fn parse_pos_token(token: &str) -> Result<Pos, PiskvorkError> {
    let [x, y]: [&str; 2] = token
        .split(',')
        .collect::<Vec<&str>>()
        .try_into()
        .map_err(|_| PiskvorkError::Error("coordinate parsing failed".to_string()))?;

    parse_pos(x, y)
}

fn parse_pos(x: &str, y: &str) -> Result<Pos, PiskvorkError> {
    let x = x.parse::<u8>().map_err(|_| PiskvorkError::Error("invalid x range.".to_string()))?;
    let y = y.parse::<u8>().map_err(|_| PiskvorkError::Error("invalid y range.".to_string()))?;

    if x < pos::BOARD_WIDTH && y < pos::BOARD_WIDTH {
        Ok(Pos::from_cartesian(y, x))
    } else {
        Err(PiskvorkError::Error("position out of range.".to_string()))
    }
}

fn parse_time(arg: Option<&str>) -> Result<u64, &'static str> {
    arg
        .ok_or("missing info value.")
        .and_then(|token| token.parse::<u64>().map_err(|_| "time parsing failed."))
}

struct Presets;

impl Presets {
    const FASTGAME: Config = Config {
        draw_condition: None,
        max_depth: None,
        max_quiescence_depth: None,

        tt_size: ByteSize::from_mib(64),
        workers: 1,
        pondering: false,
        initial_timer: Timer {
            time_unit: TimeUnit::Clock,
            total_remaining: Some(TimeValue::from_duration(Duration::from_secs(120))),
            increment: TimeValue::ZERO,
            turn: Some(TimeValue::from_duration(Duration::from_secs(5))),
        },
        spawn_depth_specialist: false,
    };

    const STANDARD: Config = Config {
        draw_condition: None,
        max_depth: None,
        max_quiescence_depth: None,

        tt_size: ByteSize::from_mib(128),
        workers: 1,
        pondering: false,
        initial_timer: Timer {
            time_unit: TimeUnit::Clock,
            total_remaining: Some(TimeValue::from_duration(Duration::from_secs(180))),
            increment: TimeValue::ZERO,
            turn: Some(TimeValue::from_duration(Duration::from_secs(30))),
        },
        spawn_depth_specialist: false,
    };

    const FINAL: Config = Config {
        draw_condition: None,
        max_depth: None,
        max_quiescence_depth: None,

        tt_size: ByteSize::from_mib(768),
        workers: 1,
        pondering: false,
        initial_timer: Timer {
            time_unit: TimeUnit::Clock,
            total_remaining: Some(TimeValue::from_duration(Duration::from_secs(1000))),
            increment: TimeValue::ZERO,
            turn: Some(TimeValue::from_duration(Duration::from_secs(300))),
        },
        spawn_depth_specialist: false,
    };
}
