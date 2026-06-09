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
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};
use mintaka::game_state::GameState;
use mintaka::protocol::timer::Timer;
use rusty_renju::notation::color::Color;
use rusty_renju::utils::empty::Empty;

const PROTOCOL_MARGIN: Duration = Duration::from_millis(30);

enum PiskvorkResponse {
    Message(String),
    Debug(String),
    Error(String),
    Unknown(String),
    About(String),
    Pos(Pos),
    Forbid(Vec<Pos>),
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
        PiskvorkResponse::Forbid(positions) => {
            println!("FORBID {}",
                 positions.iter()
                     .map(|pos| format!("{:02}{:02}", pos.col(), pos.row()))
                     .collect::<Vec<_>>()
                     .join("")
            );
        }
        PiskvorkResponse::None => {}
    };

    std::io::stdout().flush().expect("failed to flush stdout");
}

pub fn entry<const R: RuleKind>() -> Result<(), impl Error> {
    let aborted = Arc::new(AtomicBool::new(false));

    let mut config = Config::default();

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
            Message::Config(command) => {
                match command {
                    ConfigCommand::TotalTime(total) => {
                        timer.total_remaining = Some(total);
                    }
                    ConfigCommand::IncrementTime(increment) => {
                        config.initial_timer.increment = increment;
                        timer.increment = increment;
                    }
                    ConfigCommand::TurnTime(turn) => {
                        config.initial_timer.turn = Some(turn);
                        timer.turn = Some(turn)
                    }
                    ConfigCommand::Workers(workers) => {
                        config.workers = workers;
                    }
                    ConfigCommand::ResizeTT(size) => {
                        config.tt_size = size;
                        let _ = game_agent.command(Command::RebuildTT(size));
                    }
                    _ => unreachable!(),
                }
            }
            Message::Command(command) => {
                let result = game_agent.command(command.into_command(&config, game_agent.state.board.hash_key));

                if let Err(err) = result {
                    stdio_out(PiskvorkResponse::Error(err.to_string()));
                }
            }
            Message::Launch { objective, apply, .. } => {
                let best_move = game_agent.launch::<Instant>(
                    config,
                    timer,
                    objective,
                    CallBackResponseSender::new(response_receiver),
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
                        stdio_out(PiskvorkResponse::Error(err.to_string()));
                        continue;
                    }
                }

                stdio_out(PiskvorkResponse::Pos(best_move.best_move.unwrap_or(Pos::from_cartesian(7, 7))));
            }
            Message::Status(command) => match command {
                StatusCommand::Forbid => {
                    stdio_out(
                        PiskvorkResponse::Forbid(game_agent.state.board.patterns.forbidden_field.iter_hot_pos().collect())
                    );
                },
                _ => unreachable!()
            },
        }
    }

    Ok::<(), GameError>(())
}

fn spawn_command_listener<const R: RuleKind>(aborted: Arc<AtomicBool>, message_sender: MessageSender) {
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

            let piskvork_response = match_command::<R>(&aborted, &message_sender, args)
                .unwrap_or_else(|message| PiskvorkResponse::Error(message.to_string()));

            stdio_out(piskvork_response);
        }
    });
}

fn response_receiver(response: Response) {
    let piskvork_response = match response {
        Response::Begins(ComputingResource { workers, time_limit, nodes_in_1k }) =>
            PiskvorkResponse::Message(format!(
                "begins workers={workers}, running-time={time_limit:?}, nodes={nodes_in_1k:?}k"
            )),
        Response::Status { best_move, score, pv, total_nodes_in_1k, selective_depth, .. } =>
            PiskvorkResponse::Debug(format!(
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
fn match_command<const R: RuleKind>(
    aborted: &Arc<AtomicBool>,
    message_sender: &MessageSender,
    args: Vec<&str>,
) -> Result<PiskvorkResponse, &'static str> {
    let command_kind = args[0].to_uppercase();

    let response = match command_kind.as_str() {
        // basic commands
        "START" => {
            let size: usize = args
                .get(1)
                .ok_or("missing size token.")?
                .parse()
                .map_err(|_| "size parsing failed.")?;

            if size == pos::U_BOARD_WIDTH {
                message_sender.command(MessageCommand::Command(Command::Clear));

                PiskvorkResponse::Ok
            } else {
                return Err("unsupported size");
            }
        }
        "BEGIN" => {
            message_sender.launch(SearchObjective::Best, true, false);

            PiskvorkResponse::None
        }
        "INFO" => {
            match args.get(1).copied().map(str::to_lowercase).as_deref() {
                Some("timeout_match") | Some("time_left") => {
                    if let Ok(time) = parse_time(&args) {
                        message_sender.config(ConfigCommand::TotalTime(time - PROTOCOL_MARGIN));
                    }
                }
                Some("timeout_turn") => {
                    if let Ok(time) = parse_time(&args) {
                        message_sender.config(ConfigCommand::TurnTime(time - PROTOCOL_MARGIN));
                    }
                }
                Some("max_memory") => {
                    if let Some(max_memory_in_bytes) = args.get(2)
                        .and_then(|value| value.parse::<u64>().ok())
                    {
                        if max_memory_in_bytes > 10 * 1024 * 1024 {
                            message_sender.config(ConfigCommand::ResizeTT(ByteSize::from_bytes(max_memory_in_bytes)))
                        }
                    }
                }
                Some("thread_num") => {
                    if let Some(workers) = args.get(2).and_then(|value| value.parse::<u32>().ok()) {
                        message_sender.config(ConfigCommand::Workers(workers));
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
                            return Err("unsupported rule")
                        }
                    }
                }
                _ => {}
            }

            PiskvorkResponse::None
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

            PiskvorkResponse::None
        }
        "TURN" => {
            let pos = parse_command_pos(&args)?;

            message_sender.command(MessageCommand::Play { pos: pos.into() });
            message_sender.launch(SearchObjective::Best, true, false);

            PiskvorkResponse::None
        }
        "END" => {
            std::process::exit(0);
        }
        "STOP" | "YXSTOP" => {
            aborted.store(true, Ordering::Relaxed);

            PiskvorkResponse::None
        }
        // extended commands
        "RECTSTART" => return Err("rectangular board is not supported"),
        "RESTART" => {
            message_sender.command(MessageCommand::Command(Command::Clear));
            PiskvorkResponse::Ok
        }
        "TAKEBACK" => {
            parse_command_pos(&args)?;

            message_sender.command(MessageCommand::Undo);

            PiskvorkResponse::Ok
        }
        "ABOUT" => PiskvorkResponse::About(format!(
            "name=\"mintaka\", author=\"JeongHyeon Choi\", version=\"{}\", country=\"KOR\"",
            mintaka::VERSION
        )),
        "YXSHOWFORBID" => {
            message_sender.status(StatusCommand::Forbid);

            PiskvorkResponse::None
        }
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
