use mintaka::config::{Config, SearchObjective};
use mintaka::game_agent::{ComputingResource, GameAgent, GameError};
use mintaka::protocol::command::Command;
use mintaka::protocol::response::{CallBackResponseSender, Response};
use mintaka_interface::message::{ConfigCommand, Message, MessageCommand, MessageSender, StatusCommand};
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::utils::byte_size::ByteSize;
use std::error::Error;
use std::io::{BufRead, Write};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};
use mintaka::game_state::GameState;
use mintaka::protocol::timer::Timer;
use rusty_renju::notation::color::Color;
use rusty_renju::utils::empty::Empty;

pub fn entry<const R: RuleKind>() -> Result<(), impl Error> {
    piskvork_protocol::<R>()
}

const PROTOCOL_MARGIN: Duration = Duration::from_millis(30);

enum PiskvorkResponse {
    Message(String),
    Debug(String),
    Unknown(String),
    About(String),
    Pos(Pos),
    Forbid(Vec<Pos>),
    Ok,
}

fn stdio_out(piskvork_response: Result<PiskvorkResponse, String>) {
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
            println!("FORBID {}",
                 positions.iter()
                     .map(|pos| format!("{:02}{:02}", pos.col(), pos.row()))
                     .collect::<Vec<_>>()
                     .join("")
            );
        }
        Err(message) => {
            println!("ERROR {}", message);
        }
    };

    std::io::stdout().flush().expect("failed to flush stdout");
}

fn print_response(response: Response) {
    let response = match response {
        Response::Begins(ComputingResource { workers, time_limit, nodes_in_1k }) =>
            format!(
                "begins workers={workers}, running-time={time_limit:?}, nodes={nodes_in_1k:?}k"
            ),
        Response::Status { best_move, score, pv, total_nodes_in_1k, selective_depth, .. } =>
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

    let mut config = Presets::FASTGAME_PRESET;

    let mut game_agent = GameAgent::<R>::new(config);

    let mut timer = Timer {
        total_remaining: Some(Duration::from_secs(180)),
        increment: Duration::ZERO,
        turn: Some(Duration::from_secs(30)),
    };

    let (message_sender, message_receiver) = {
        let (tx, rx) = mpsc::channel();
        (MessageSender::new(tx), rx)
    };

    spawn_command_listener::<R>(aborted.clone(), message_sender);

    for message in message_receiver {
        match message {
            Message::Command(command) => {
                let result = game_agent.command(command.into_command(&config, game_agent.state.board.hash_key));

                if let Err(err) = result {
                    stdio_out(Err(err.to_string()));
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

                    if let Err(err) = result {
                        stdio_out(Err(err.to_string()));
                        continue;
                    }
                }

                stdio_out(Ok(PiskvorkResponse::Pos(best_move.best_move.unwrap_or(Pos::from_cartesian(7, 7)))));
            }
            Message::Config(ConfigCommand::TotalTime(total)) => {
                timer.total_remaining = Some(total);
            }
            Message::Config(ConfigCommand::IncrementTime(increment)) => {
                config.initial_timer.increment = increment;
                timer.increment = increment;
            }
            Message::Config(ConfigCommand::TurnTime(turn)) => {
                config.initial_timer.turn = Some(turn);
                timer.turn = Some(turn)
            }
            Message::Config(ConfigCommand::MaxNodes { in_1k }) => {
                config.max_nodes_in_1k = Some(in_1k);
            }
            Message::Config(ConfigCommand::Workers(workers)) => {
                config.workers = workers;
            }
            Message::Config(ConfigCommand::ResizeTT(size)) => {
                config.tt_size = size;
                let _ = game_agent.command(Command::RebuildTT(config.tt_size));
            }
            Message::Config(_) => unreachable!(),
            Message::Status(StatusCommand::Forbid) => {
                stdio_out(Ok(PiskvorkResponse::Forbid(
                    game_agent.state.board.patterns.forbidden_field.iter_hot_pos().collect()
                )));
            }
            Message::Status(StatusCommand::Version) => {
                stdio_out(Ok(PiskvorkResponse::About(
                    format!(
                        "name=\"mintaka\", author=\"JeongHyeon Choi\", version=\"{}\", country=\"KOR\"",
                        mintaka::VERSION
                    )
                )));
            }
            Message::Status(_) => unreachable!()
        }
    }

    Ok::<(), GameError>(())
}

// https://plastovicka.github.io/protocl2en.htm
// https://github.com/accreator/Yixin-protocol/blob/master/protocol.pdf
fn match_command<const R: RuleKind>(
    aborted: &Arc<AtomicBool>,
    message_sender: &MessageSender,
    args: Vec<&str>,
) -> Result<(), &'static str> {
    let command_kind = args[0].to_uppercase();

    match command_kind.as_str() {
        // basic commands
        "START" => {
            let size: usize = args
                .get(1)
                .ok_or("missing size token.")?
                .parse()
                .map_err(|_| "size parsing failed.")?;

            if size == pos::U_BOARD_WIDTH {
                message_sender.command(MessageCommand::Command(Command::Clear));
            } else {
                return Err("unsupported size");
            }
        }
        "BEGIN" => {
            message_sender.launch(SearchObjective::Best, true, false);
        }
        "INFO" => {
            match args.get(1).copied().map(str::to_lowercase).as_deref() {
                Some("timeout_match") | Some("time_left") => {
                    if let Ok(time) = parse_time(&args) {
                        message_sender.config(ConfigCommand::TotalTime(time - PROTOCOL_MARGIN));
                    } else {
                        return Err("invalid time value");
                    }
                }
                Some("timeout_turn") => {
                    if let Ok(time) = parse_time(&args) {
                        message_sender.config(ConfigCommand::TurnTime(time - PROTOCOL_MARGIN));
                    } else {
                        return Err("invalid time value");
                    }
                }
                Some("max_memory") => {
                    if let Some(max_memory_in_bytes) = args.get(2)
                        && let Some(max_memory_in_bytes) = max_memory_in_bytes.parse::<u64>().ok()
                        && max_memory_in_bytes > 10 * 1024 * 1024
                    {
                        message_sender.config(ConfigCommand::ResizeTT(ByteSize::from_bytes(max_memory_in_bytes)));
                    } else {
                        return Err("invalid memory value");
                    }
                }
                Some("thread_num") => {
                    if let Some(workers) = args.get(2).and_then(|value| value.parse::<u32>().ok()) {
                        message_sender.config(ConfigCommand::Workers(workers));
                    } else {
                        return Err("invalid thread value");
                    }
                }
                Some("game_type") => {
                    let _ = args.get(2);
                }
                Some("rule") => {
                    if let Some(rule) = args.get(2).and_then(|value| value.parse::<usize>().ok()) {
                        let rule_kind = match rule {
                            1 => RuleKind::Gomoku,
                            2 | 4 => RuleKind::Renju,
                            _ => return Err("unsupported rule"),
                        };

                        if rule_kind != R {
                            return Err("unsupported rule");
                        }
                    } else {
                        return Err("invalid rule value");
                    }
                }
                _ => return Err("unknown info token"),
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
                    .map_err(|_| "failed to stdio")?;

                if buf.trim() == DONE_TOKEN {
                    break;
                }

                let [x, y, color]: [&str; 3] = buf
                    .trim()
                    .split(',')
                    .collect::<Vec<&str>>()
                    .try_into()
                    .map_err(|_| "token parsing failed")?;

                let pos = parse_pos(x, y)?;

                match color {
                    "1" => sequence.push((pos, true)),
                    "2" => sequence.push((pos, false)),
                    "3" => {},
                    &_ => return Err("unknown color token")
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
                    return Err("illegal move");
                }

                game_state.play_mut(pos);
            }

            message_sender.command(MessageCommand::Command(Command::Init(Box::new((&game_state).into()))));

            if command_kind.as_str() == "BOARD" {
                message_sender.launch(SearchObjective::Best, true, false);
            }
        }
        "TURN" => {
            let pos = parse_command_pos(&args)?;

            message_sender.command(MessageCommand::Play { pos: pos.into() });
            message_sender.launch(SearchObjective::Best, true, false);
        }
        "END" => {
            std::process::exit(0);
        }
        "STOP" | "YXSTOP" => {
            aborted.store(true, Ordering::Relaxed);
        }
        // extended commands
        "RECTSTART" => return Err("rectangular board is not supported"),
        "RESTART" => {
            message_sender.command(MessageCommand::Command(Command::Clear));
        }
        "TAKEBACK" => {
            parse_command_pos(&args)?;

            message_sender.command(MessageCommand::Undo);
        }
        "ABOUT" => {
            message_sender.status(StatusCommand::Version);
        },
        "YXSHOWFORBID" => {
            message_sender.status(StatusCommand::Forbid);
        }
        &_ => return Err("unknown command."),
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
            let args = line.trim().split(' ').collect::<Vec<&str>>();

            if args.is_empty() {
                continue;
            }

            let result = match_command::<R>(&aborted, &message_sender, args);

            if let Err(error) = result {
                stdio_out(Err(error.to_string()));
            }
        }
    });
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

struct Presets;

impl Presets {
    const FASTGAME_PRESET: Config = Config {
        draw_condition: None,
        max_nodes_in_1k: None,
        max_depth: None,
        max_vcf_depth: None,

        tt_size: ByteSize::from_mib(64),
        workers: 1,
        pondering: false,
        initial_timer: Timer {
            total_remaining: Some(Duration::from_secs(120)),
            increment: Duration::ZERO,
            turn: Some(Duration::from_secs(5)),
        },
        spawn_depth_specialist: false,
    };

    const STANDARD_PRESET: Config = Config {
        draw_condition: None,
        max_nodes_in_1k: None,
        max_depth: None,
        max_vcf_depth: None,

        tt_size: ByteSize::from_mib(128),
        workers: 1,
        pondering: false,
        initial_timer: Timer {
            total_remaining: Some(Duration::from_secs(180)),
            increment: Duration::ZERO,
            turn: Some(Duration::from_secs(30)),
        },
        spawn_depth_specialist: false,
    };

    const FINAL_PRESET: Config = Config {
        draw_condition: None,
        max_nodes_in_1k: None,
        max_depth: None,
        max_vcf_depth: None,

        tt_size: ByteSize::from_mib(768),
        workers: 1,
        pondering: false,
        initial_timer: Timer {
            total_remaining: Some(Duration::from_secs(1000)),
            increment: Duration::ZERO,
            turn: Some(Duration::from_secs(300)),
        },
        spawn_depth_specialist: false,
    };
}
