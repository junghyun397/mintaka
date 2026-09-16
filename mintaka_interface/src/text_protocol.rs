use mintaka::config::{Config, SearchObjective};
use mintaka::game_agent::{ComputingResource, GameAgent, GameError};
use mintaka::game_state::{GameState, GameStateData};
use mintaka::protocol::command::Command;
use mintaka::protocol::response::{CallBackResponseSender, Response};
use mintaka::protocol::time::{TimeUnit, TimeValue};
use mintaka::utils::depth::Depth;
use mintaka_interface::message::{ConfigCommand, Message, MessageCommand, MessageSender, StatusCommand};
use mintaka_interface::params::Params;
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::notation::color::UnknownColorError;
use rusty_renju::notation::pos::{MaybePos, PosError};
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::utils::byte_size::ByteSize;
use rusty_renju::utils::empty::Empty;
use std::io::{BufRead, Write};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, mpsc};
use std::time::Instant;

pub fn entry<const R: RuleKind>() -> Result<(), GameError> {
    let params = Params::<R>::parse();

    let command_sequence: Vec<String> = params
        .command_sequence
        .map(|sequence| {
            sequence
                .split('\n')
                .filter(|&line| !line.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    text_protocol(
        params.config,
        params.game_state.unwrap_or_else(|| GameState::empty()),
        command_sequence,
    )
}

enum TextProtocolResponse {
    Log(String),
    Response(String),
    Ack,
    Multiline(String),
}

fn stdio_out(text_protocol_response: Result<TextProtocolResponse, String>) {
    match text_protocol_response { 
        Ok(TextProtocolResponse::Log(message)) => {
            println!("% {}", message);
        }
        Ok(TextProtocolResponse::Response(response)) => {
            println!("= {}", response);
        }
        Ok(TextProtocolResponse::Ack) => {
            println!("=");
        }
        Ok(TextProtocolResponse::Multiline(content)) => {
            println!("=\x02\n{}\x03", content);
        }
        Err(error) => {
            println!("? {}", error);
        }
    }

    std::io::stdout().flush().expect("failed to flush stdout");
}

fn print_response(response: Response) {
    let log = match response {
        Response::Begins(ComputingResource { workers, time_unit, time_limit }) =>
            format!("begins: workers={workers}, time-unit={time_unit}, resource={time_limit:?}"),
        Response::Status { best_move, score, pv, total_nodes: total_nodes_in_1k, selective_depth, .. } =>
            format!("status: depth={selective_depth}, score={score}, best_move={best_move}, total_nodes_in_1k={total_nodes_in_1k}, pv={pv:?}"),
    };

    stdio_out(Ok(TextProtocolResponse::Log(log)));
}

fn text_protocol<const R: RuleKind>(
    mut config: Config,
    state: GameState<R>,
    command_sequence: Vec<String>,
) -> Result<(), GameError> {
    let aborted = Arc::new(AtomicBool::new(false));

    let mut game_agent = GameAgent::from_state(config, state);

    let mut timer = config.initial_timer;

    let (message_sender, message_receiver) = {
        let (tx, rx) = mpsc::channel();
        (MessageSender::new(tx), rx)
    };

    spawn_command_listener::<R>(aborted.clone(), message_sender, command_sequence);

    for message in message_receiver {
        match message {
            Message::Command(command) => {
                let command = command.into_command(&config, game_agent.state.board.hash_key);

                let response = execute_command(&mut game_agent, command);

                stdio_out(response);
            }
            Message::Launch { objective, apply, print: interactive } => {
                let best_move = game_agent.launch::<Instant>(
                    config,
                    timer,
                    objective,
                    CallBackResponseSender::new(print_response),
                    Arc::new(AtomicU32::new(0)),
                    aborted.clone(),
                );

                stdio_out(Ok(TextProtocolResponse::Response(
                    format!(
                        "pos={}, score={}, depth={}, nodes={}, elapsed={:?}",
                        best_move.best_move,
                        best_move.score,
                        best_move.selective_depth,
                        best_move.total_nodes,
                        best_move.time_elapsed,
                    )
                )));

                if apply {
                    let command = Command::Play {
                        hash: game_agent.state.board.hash_key,
                        pos: best_move.best_move.into(),
                        draw_condition: config.draw_condition,
                    };

                    let response = execute_command(&mut game_agent, command);

                    stdio_out(response);
                }

                if interactive {
                    stdio_out(Ok(TextProtocolResponse::Multiline(format_board(&game_agent.state, true))))
                }
            }
            Message::Config(ConfigCommand::TimeUnit(unit)) => {
                timer.time_unit = unit;

                stdio_out(Ok(TextProtocolResponse::Ack));
            }
            Message::Config(ConfigCommand::TotalTime(total)) => {
                timer.total_remaining = Some(TimeValue::from_value(total, timer.time_unit));

                stdio_out(Ok(TextProtocolResponse::Ack));
            }
            Message::Config(ConfigCommand::IncrementTime(increment)) => {
                let increment = TimeValue::from_value(increment, timer.time_unit);

                config.initial_timer.increment = increment;
                timer.increment = increment;

                stdio_out(Ok(TextProtocolResponse::Ack));
            }
            Message::Config(ConfigCommand::TurnTime(turn)) => {
                let turn = TimeValue::from_value(turn, timer.time_unit);

                config.initial_timer.turn = Some(turn);
                timer.turn = Some(turn);

                stdio_out(Ok(TextProtocolResponse::Ack));
            }
            Message::Config(ConfigCommand::MaxDepth(max_depth)) => {
                config.max_depth = Some(Depth::from_i32(max_depth as i32));

                stdio_out(Ok(TextProtocolResponse::Ack));
            }
            Message::Config(ConfigCommand::Workers(workers)) => {
                config.workers = workers;

                stdio_out(Ok(TextProtocolResponse::Ack));
            }
            Message::Config(ConfigCommand::ResizeTT(size)) => {
                config.tt_size = size;

                let _ = game_agent.command(Command::RebuildTT(config.tt_size));

                stdio_out(Ok(TextProtocolResponse::Ack));
            }
            Message::Status(StatusCommand::Version) => {
                stdio_out(Ok(TextProtocolResponse::Response(
                    format!(
                        "rule={}, rusty-renju={}, mintaka={}",
                        R, rusty_renju::VERSION, mintaka::VERSION
                    )
                )));
            }
            Message::Status(StatusCommand::Board { show_last_moves }) => {
                stdio_out(Ok(TextProtocolResponse::Multiline(format_board(&game_agent.state, show_last_moves))));
            }
            Message::Status(StatusCommand::History) => {
                stdio_out(Ok(TextProtocolResponse::Response(
                    game_agent.state.history.to_string()
                )));
            }
            Message::Status(StatusCommand::Time) => {
                stdio_out(Ok(TextProtocolResponse::Response(
                    format!("total={}, increment={}, turn={}",
                        format_time_value(timer.time_unit, timer.total_remaining),
                        format_time_value(timer.time_unit, Some(timer.increment)),
                        format_time_value(timer.time_unit, timer.turn),
                    )
                )))
            },
            Message::Status(StatusCommand::Forbid) => unreachable!(),
        }
    }

    Ok(())
}

fn match_command<const R: RuleKind>(
    aborted: &Arc<AtomicBool>,
    message_sender: &MessageSender,
    args: Vec<&str>,
    buf: &str,
) -> Result<(), String> {
    match args[0] {
        "abort" => {
            aborted.store(true, Ordering::Relaxed);
        }
        "quit" => {
            std::process::exit(0);
        }
        "workers" => match *args.get(1).ok_or("workers not provided.".to_string())? {
            "auto" => {
                let cores =
                    std::thread::available_parallelism().map_or_else(|_| 1, |n| n.get()) as u32;

                message_sender.config(ConfigCommand::Workers(cores));
            }
            &_ => {
                let workers = args.get(1).ok_or("workers not provided.")?
                    .parse::<u32>()
                    .ok()
                    .filter(|&workers| workers > 0)
                    .ok_or("invalid workers number.")?;

                message_sender.config(ConfigCommand::Workers(workers));
            }
        },
        "memory" => {
            let memory_size_in_kib = args.get(1).ok_or("memory not provided.")?
                .parse::<u64>()
                .map_err(|_| "invalid memory size.")?;

            message_sender.config(ConfigCommand::ResizeTT(ByteSize::from_kib(memory_size_in_kib)));
        }
        "time" => {
            let parse_time = || -> Result<u64, &'static str> {
                args.get(2).ok_or("time not provided.")?
                    .parse::<u64>()
                    .map_err(|_| "invalid time.")
            };

            match *args.get(1).ok_or("data type not provided.")? {
                "unit" => {
                    let time_unit = args.get(2).ok_or("time unit not provided.")?
                        .parse::<TimeUnit>()
                        .map_err(|_| "invalid time unit.")?;

                    message_sender.config(ConfigCommand::TimeUnit(time_unit));
                }
                "total" => {
                    message_sender.config(ConfigCommand::TotalTime(parse_time()?));
                }
                "turn" => {
                    message_sender.config(ConfigCommand::TurnTime(parse_time()?));
                }
                "increment" => {
                    message_sender.config(ConfigCommand::IncrementTime(parse_time()?));
                }
                "left" => {
                    message_sender.status(StatusCommand::Time);
                }
                &_ => return Err("unknown time type.".to_string()),
            }
        }
        "load" => match *args.get(1).ok_or("data type not provided.")? {
            "board" => {
                let board: Board<R> = buf.parse()?;

                let history = (&board).try_into().unwrap_or_else(|_| History::empty());

                message_sender.command(MessageCommand::Command(Command::Init(Box::new(GameStateData { board_data: (&board).into(), history }))));
            }
            "history" => {
                let history: History = args.get(2).ok_or("history not provided.")?.parse()?;

                let board: Board<R> = (&history).into();

                message_sender.command(MessageCommand::Command(Command::Init(Box::new(GameStateData { board_data: (&board).into(), history }))));
            }
            &_ => return Err("unknown data type.".to_string()),
        },
        "clear" => {
            message_sender.command(MessageCommand::Command(Command::Clear));
        }
        "board" => {
            message_sender.status(StatusCommand::Board {
                show_last_moves: false,
            });
        }
        "history" => {
            message_sender.status(StatusCommand::History);
        }
        "version" => {
            message_sender.status(StatusCommand::Version);
        }
        "set" => {
            let pos = args.get(1).ok_or("position not provided.")?
                .parse()
                .map_err(|e: PosError| e.to_string())?;

            let color = args.get(2).ok_or("color not provided.")?
                .parse()
                .map_err(|e: UnknownColorError| e.to_string())?;

            message_sender.command(MessageCommand::Set { pos, color });
        }
        "unset" => {
            let pos = args.get(1).ok_or("position not provided.")?
                .parse()
                .map_err(|e: PosError| e.to_string())?;

            let color = args.get(2).ok_or("color not provided.")?
                .parse()
                .map_err(|e: UnknownColorError| e.to_string())?;

            message_sender.command(MessageCommand::Unset { pos, color });
        }
        "play" => {
            let action: MaybePos = args.get(1).ok_or("position not provided.")?
                .parse()
                .map_err(|e: PosError| e.to_string())?;

            message_sender.command(MessageCommand::Play { pos: action });
        }
        "undo" => {
            message_sender.command(MessageCommand::Undo);
        }
        "gen" => {
            message_sender.launch(SearchObjective::Best, false, false);
        }
        "igen" => {
            message_sender.launch(SearchObjective::Best, true, true);
        }
        "zero" => {
            message_sender.launch(SearchObjective::Zeroing, true, true);
        }
        "ponder" => {
            message_sender.launch(SearchObjective::Pondering, false, false);
        }
        &_ => return Err("unknown command.".to_string()),
    };

    Ok(())
}

fn execute_command<const R: RuleKind>(game_agent: &mut GameAgent<R>, command: Command) -> Result<TextProtocolResponse, String> {
    let result = game_agent.command(command);

    result
        .map(|command_result| command_result.result
            .map(|game_result| TextProtocolResponse::Response(game_result.to_string()))
            .unwrap_or_else(|| TextProtocolResponse::Ack)
        )
        .map_err(|err| err.to_string())
}

fn spawn_command_listener<const R: RuleKind>(
    aborted: Arc<AtomicBool>,
    message_sender: MessageSender,
    initial_sequence: Vec<String>,
) {
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let stdin_lines = stdin.lock().lines();

        for line in initial_sequence
            .into_iter()
            .chain(stdin_lines.map(Result::unwrap))
        {
            let args = line.trim().split(' ').collect::<Vec<&str>>();

            if args.is_empty() {
                continue;
            }

            let result = match_command::<R>(&aborted, &message_sender, args, &line);

            if let Err(error) = result {
                stdio_out(Err(error));
            }
        }
    });
}

fn format_board<const R: RuleKind>(state: &GameState<R>, show_last_moves: bool) -> String {
    if show_last_moves {
        state.board.to_string_with_last_moves(state.history.last_action_pair())
    } else {
        state.board.to_string()
    }
}

fn format_time_value(unit: TimeUnit, value: Option<TimeValue>) -> String {
    match (unit, value) {
        (TimeUnit::Clock, Some(value)) => {
            let duration = value.to_duration();

            format!("{}.{}s", duration.as_secs(), duration.subsec_millis())
        },
        (TimeUnit::Nodes, Some(value)) => format!("{}K nodes", value.to_nodes()),
        _ => "infinite".to_string()
    }
}
