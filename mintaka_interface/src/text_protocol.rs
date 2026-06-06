use mintaka::config::{Config, SearchObjective};
use mintaka::game_agent::{ComputingResource, GameAgent, GameError};
use mintaka::game_state::{GameState, GameStateData};
use mintaka::protocol::command::Command;
use mintaka::protocol::response::{CallBackResponseSender, Response};
use mintaka_interface::message::{ConfigCommand, Message, MessageCommand, MessageSender, StatusCommand};
use mintaka_interface::preference::Preference;
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::notation::color::UnknownColorError;
use rusty_renju::notation::pos::{MaybePos, PosError};
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::utils::byte_size::ByteSize;
use rusty_renju::utils::empty::Empty;
use std::io::BufRead;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};
use mintaka::value::Depth;

pub fn entry<const R: RuleKind>() -> Result<(), GameError> {
    let pref = Preference::<R>::parse();

    let command_sequence: Vec<String> = pref
        .command_sequence
        .map(|sequence| {
            sequence
                .split('\n')
                .filter(|&line| !line.is_empty())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default();

    text_protocol(
        pref.config,
        pref.game_state.unwrap_or_else(|| GameState::empty()),
        command_sequence,
    )
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
                    ConfigCommand::MaxNodes { in_1k } => {
                        config.max_nodes_in_1k = Some(in_1k);
                    }
                    ConfigCommand::MaxDepth(max_depth) => {
                        config.max_depth = Some(max_depth as Depth);
                    }
                    ConfigCommand::Workers(workers) => {
                        config.workers = workers;
                    }
                    ConfigCommand::ResizeTT(size) => {
                        config.tt_size = size;
                        let _ = game_agent.command(Command::RebuildTT(config.tt_size));
                    }
                }
            }
            Message::Command(command) => {
                let command = command.into_command(&config, game_agent.state.board.hash_key);

                execute_command(&mut game_agent, command);
            }
            Message::Status(command) => print_status(&game_agent, command),
            Message::Launch { objective, apply, interactive } => {
                let best_move = game_agent.launch::<Instant>(
                    config,
                    timer,
                    objective,
                    CallBackResponseSender::new(print_response),
                    Arc::new(AtomicU32::new(0)),
                    aborted.clone(),
                );

                println!(
                    "solution: pos={}, score={}, depth={}, nodes={}k, elapsed={:?}",
                    best_move.best_move,
                    best_move.score,
                    best_move.selective_depth,
                    best_move.total_nodes_in_1k,
                    best_move.time_elapsed,
                );

                println!("= {}", best_move.best_move);

                if apply {
                    let command = Command::Play {
                        hash: game_agent.state.board.hash_key,
                        pos: best_move.best_move.into(),
                        draw_condition: config.draw_condition,
                    };

                    execute_command(&mut game_agent, command);
                }

                if interactive {
                    print_status(&game_agent, StatusCommand::Board { show_last_moves: true });
                }
            }
        }
    }

    Ok(())
}

fn execute_command<const R: RuleKind>(game_agent: &mut GameAgent<R>, command: Command) {
    match game_agent.command(command) {
        Ok(result) => match result.result {
            Some(result) => println!("= {result}"),
            None => println!("="),
        },
        Err(err) => println!("? {err}"),
    }
}

fn print_status<const R: RuleKind>(game_agent: &GameAgent<R>, command: StatusCommand) {
    match command {
        StatusCommand::Version => println!(
            "= rule-{}, rusty-renju-{}, mintaka-{}",
            R, rusty_renju::VERSION, mintaka::VERSION
        ),
        StatusCommand::Board { show_last_moves: false } =>
            println!("=\x02\n{}\x03", game_agent.state.board),
        StatusCommand::Board { show_last_moves: true } => println!(
            "=\x02\n{}\x03",
            game_agent
                .state
                .board
                .to_string_with_last_moves(game_agent.state.history.last_action_pair())
        ),
        StatusCommand::History => println!("= {}", game_agent.state.history),
        _ => {}
    }
}

fn print_response(response: Response) {
    match response {
        Response::Begins(ComputingResource { workers, time_limit, nodes_in_1k }) =>
            println!("begins: workers={workers}, running-time={time_limit:?}, nodes={nodes_in_1k:?}"),
        Response::Status { best_move, score, pv, total_nodes_in_1k, selective_depth, .. } =>
            println!("status: depth={selective_depth}, score={score}, best_move={best_move}, total_nodes_in_1k={total_nodes_in_1k}, pv={pv:?}"),
    }
}

fn spawn_command_listener<const R: RuleKind>(
    abort: Arc<AtomicBool>,
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

            match handle_command::<R>(&abort, &message_sender, &line, args) {
                Err(err) => println!("? {err}"),
                _ => {}
            }
        }
    });
}

fn handle_command<const R: RuleKind>(
    abort: &Arc<AtomicBool>,
    message_sender: &MessageSender,
    buf: &str,
    args: Vec<&str>,
) -> Result<(), String> {
    match args[0].to_ascii_lowercase().as_str() {
        "abort" => {
            abort.store(true, Ordering::Relaxed);
        }
        "quit" => {
            std::process::exit(0);
        }
        "config" => match *args.get(1).ok_or("data type not provided.".to_string())? {
            "workers" => match *args.get(2).ok_or("workers not provided.".to_string())? {
                "auto" => {
                    let cores =
                        std::thread::available_parallelism().map_or_else(|_| 1, |n| n.get()) as u32;

                    println!("info: workers={cores}");

                    message_sender.config(ConfigCommand::Workers(cores));
                }
                &_ => {
                    let workers = args.get(2).ok_or("workers not provided.")?
                        .parse::<u32>()
                        .ok()
                        .filter(|&workers| workers > 0)
                        .ok_or("invalid workers number.")?;

                    message_sender.config(ConfigCommand::Workers(workers));
                }
            },
            "memory" => {
                let memory_size_in_kib = args.get(2).ok_or("memory not provided.")?
                    .parse::<u64>()
                    .map_err(|_| "invalid memory size.")?;

                message_sender.config(ConfigCommand::ResizeTT(ByteSize::from_kib(memory_size_in_kib)));
            }
            &_ => return Err("data type not provided.".to_string()),
        },
        "limit" => match *args.get(1).ok_or("data type not provided.")? {
            "time" => {
                fn parse_time_in_milliseconds(args: &Vec<&str>) -> Result<Duration, &'static str> {
                    let time = args.get(3).ok_or("time not provided.")?
                        .parse::<u64>()
                        .map_err(|_| "invalid time.")?;

                    Ok(Duration::from_millis(time))
                }

                match *args.get(2).ok_or("data type not provided.")? {
                    "total" => {
                        message_sender.config(ConfigCommand::TotalTime(parse_time_in_milliseconds(&args)?));
                    }
                    "turn" => {
                        message_sender.config(ConfigCommand::TurnTime(parse_time_in_milliseconds(&args)?));
                    }
                    "increment" => {
                        message_sender.config(ConfigCommand::IncrementTime(parse_time_in_milliseconds(&args)?));
                    }
                    &_ => return Err("unknown time type.".to_string()),
                }
            }
            "nodes" => {
                let nodes = args.get(1).ok_or("nodes not provided.")?
                    .parse::<u32>()
                    .map_err(|_| "invalid nodes number.")?;

                message_sender.config(ConfigCommand::MaxNodes { in_1k: nodes });
            }
            &_ => return Err("unknown limit type.".to_string()),
        },
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
        "time" => {
            message_sender.status(StatusCommand::Time);
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
        &_ => return Err("unknown command.".to_string()),
    }

    Ok(())
}
