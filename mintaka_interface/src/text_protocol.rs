use mintaka::config::{Config, SearchObjective};
use mintaka::game_agent::{ComputingResource, GameAgent, GameError};
use mintaka::game_state::GameState;
use mintaka::protocol::command::Command;
use mintaka::protocol::response::{CallBackResponseSender, Response};
use mintaka_interface::message::{Message, MessageSender, StatusCommand};
use mintaka_interface::preference::Preference;
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::notation::color::UnknownColorError;
use rusty_renju::notation::pos::{Pos, PosError};
use rusty_renju::utils::byte_size::ByteSize;
use std::io::BufRead;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

fn main() -> Result<(), GameError> {
    let pref = Preference::parse();

    let command_sequence: Vec<String> = pref.command_sequence
        .map(|sequence|
            sequence
                .split('\n')
                .filter(|&line| !line.is_empty())
                .map(String::from)
                .collect()
        )
        .unwrap_or_default();

    text_protocol(pref.default_config, pref.game_state.unwrap_or_default(), command_sequence)
}

fn text_protocol(config: Config, state: GameState, command_sequence: Vec<String>) -> Result<(), GameError> {
    let launched = Arc::new(AtomicBool::new(false));
    let aborted = Arc::new(AtomicBool::new(false));

    let mut game_agent = GameAgent::from_state(config, state);

    let (message_sender, message_receiver) = {
        let (tx, rx) = mpsc::channel();
        (MessageSender::new(tx), rx)
    };

    spawn_command_listener(launched.clone(), aborted.clone(), message_sender.clone(), command_sequence);

    for message in message_receiver {
        match message {
            Message::Ok => {
                println!("=")
            }
            Message::Command(command) => {
                match game_agent.command(command) {
                    Ok(result) => message_sender.result(result),
                    Err(err) => {
                        println!("? {err}");
                        continue;
                    }
                }
            },
            Message::Status(command) => match command {
                StatusCommand::Version =>
                    println!("= rusty-renju-{}, mintaka-{}, unknown", rusty_renju::VERSION, mintaka::VERSION),
                StatusCommand::Board { show_last_moves: false } =>
                    println!("=\x02\n{}\x03", game_agent.state.board),
                StatusCommand::Board { show_last_moves: true } =>
                    println!("=\x02\n{}\x03", game_agent.state.board.to_string_with_last_moves(game_agent.state.history.recent_action_pair())),
                StatusCommand::History =>
                    println!("= {}", game_agent.state.history),
                StatusCommand::Time =>
                    println!("= {:?}", game_agent.time_manager.timer),
            },
            Message::Finished(result) => {
                println!("= {result}")
            }
            Message::Launch { objective, apply, interactive } => {
                launched.store(true, Ordering::Relaxed);

                let best_move = game_agent.launch::<Instant>(
                    objective,
                    CallBackResponseSender::new(response_printer),
                    aborted.clone(),
                );

                launched.store(false, Ordering::Relaxed);

                println!(
                    "solution: pos={}, score={}, depth={}, nodes={}k, elapsed={:?}",
                    best_move.pos, best_move.score, best_move.selective_depth, best_move.total_nodes_in_1k, best_move.time_elapsed
                );

                println!("= {}", best_move.pos);

                if apply {
                    message_sender.command(Command::Play(best_move.pos.into()));
                }

                if interactive {
                    message_sender.status(StatusCommand::Board { show_last_moves: true });
                }
            }
        }
    }

    Ok(())
}

fn response_printer(response: Response) {
    match response {
        Response::Begins(ComputingResource { workers, time, nodes_in_1k, tt_size }) =>
            println!("begins: workers={workers}, \
                running-time={time:?}, \
                nodes={nodes_in_1k:?}, \
                tt-size={tt_size}"),
        Response::Status { best_move, score, pv, total_nodes_in_1k, depth } =>
            println!(
                "status: depth={depth}, \
                score={score}, \
                best_move={best_move}, \
                total_nodes_in_1k={total_nodes_in_1k}, \
                pv={pv:?}"
            ),
        _ => {}
    }
}

fn spawn_command_listener(launched: Arc<AtomicBool>, abort: Arc<AtomicBool>, message_sender: MessageSender, initial_sequence: Vec<String>) {
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let stdin_lines = stdin.lock().lines();

        for line in initial_sequence.into_iter()
            .chain(stdin_lines.map(Result::unwrap))
        {
            let args = line.trim().split(' ').collect::<Vec<&str>>();

            if args.is_empty() {
                continue;
            }

            match handle_command(&launched, &abort, &message_sender, &line, args) {
                Err(err) => println!("? {err}"),
                _ => {}
            }
        }
    });
}

fn handle_command(
    launched: &Arc<AtomicBool>, abort: &Arc<AtomicBool>, message_sender: &MessageSender, buf: &str, args: Vec<&str>
) -> Result<(), String> {
    if launched.load(Ordering::Relaxed) {
        match args[0] {
            "abort" => {
                abort.store(true, Ordering::Relaxed);
            },
            "quit" => {
                std::process::exit(0);
            },
            &_ => return Err("unknown command.".to_string())
        }
    } else {
        match args[0] {
            "config" => {
                match *args.get(1)
                    .ok_or("data type not provided.".to_string())?
                {
                    "workers" => {
                        match *args.get(2)
                            .ok_or("workers not provided.".to_string())?
                        {
                            "auto" => {
                                let cores = num_cpus::get_physical() as u32;

                                println!("info: workers={cores}");

                                message_sender.command(Command::Workers(cores));
                            },
                            &_ => {
                                let workers = args.get(2)
                                    .ok_or("workers not provided.")?
                                    .parse::<u32>()
                                    .ok()
                                    .filter(|&workers| workers > 0)
                                    .ok_or("invalid workers number.")?;

                                message_sender.command(Command::Workers(workers));
                            }
                        }
                    },
                    "memory" => {
                        let memory_size_in_kib = args.get(2)
                            .ok_or("memory not provided.")?
                            .parse::<u64>()
                            .map_err(|_| "invalid memory size.")?;

                        message_sender.command(Command::MaxMemory(ByteSize::from_kib(memory_size_in_kib)));
                    },
                    &_ => return Err("data type not provided.".to_string())
                }
            },
            "limit" => {
                match *args.get(1)
                    .ok_or("data type not provided.")?
                {
                    "time" => {
                        fn parse_time_in_milliseconds(args: &Vec<&str>) -> Result<Duration, &'static str> {
                            let time = args.get(3)
                                .ok_or("time not provided.")?
                                .parse::<u64>()
                                .map_err(|_| "invalid time.")?;

                            Ok(Duration::from_millis(time))
                        }

                        match *args.get(2)
                            .ok_or("data type not provided.")?
                        {
                            "total" => {
                                message_sender.command(
                                    Command::TotalTime(parse_time_in_milliseconds(&args)?)
                                );
                            },
                            "turn" => {
                                message_sender.command(
                                    Command::TurnTime(parse_time_in_milliseconds(&args)?)
                                );
                            },
                            "increment" => {
                                message_sender.command(
                                    Command::IncrementTime(parse_time_in_milliseconds(&args)?)
                                );
                            }
                            &_ => return Err("unknown time type.".to_string())
                        }
                    },
                    "nodes" => {
                        let nodes = args.get(1)
                            .ok_or("nodes not provided.")?
                            .parse::<u64>()
                            .map_err(|_| "invalid nodes number.")?;

                        message_sender.command(Command::MaxNodes { in_1k: nodes });
                    },
                    &_ => return Err("unknown limit type.".to_string()),
                }
            }
            "load" => {
                match *args.get(1)
                    .ok_or("data type not provided.")?
                {
                    "board" => {
                        message_sender.command(Command::Load(
                            Box::new((buf.parse()?, History::default()))
                        ));
                    },
                    "history" => {
                        let history: History = args.get(2)
                            .ok_or("history not provided.")?
                            .parse()?;

                        let mut board = Board::default();

                        board.batch_set_mut(history.actions());

                        message_sender.command(Command::Load(
                            Box::new((board, history))
                        ));
                    },
                    &_ => return Err("unknown data type.".to_string()),
                }
            },
            "clear" => {
                message_sender.command(Command::Clear);
            },
            "board" => {
                message_sender.status(StatusCommand::Board { show_last_moves: false });
            },
            "history" => {
                message_sender.status(StatusCommand::History);
            },
            "time" => {
                message_sender.status(StatusCommand::Time);
            }
            "version" => {
                message_sender.status(StatusCommand::Version);
            },
            "set" => {
                let pos = args.get(1).ok_or("position not provided.")?
                    .parse()
                    .map_err(|e: PosError| e.to_string())?;
                let color = args.get(2).ok_or("color not provided.")?
                    .parse()
                    .map_err(|e: UnknownColorError| e.to_string())?;

                message_sender.command(Command::Set { pos, color });
            },
            "unset" => {
                let pos = args.get(1).ok_or("position not provided.")?
                    .parse()
                    .map_err(|e: PosError| e.to_string())?;
                let color = args.get(2).ok_or("color not provided.")?
                    .parse()
                    .map_err(|e: UnknownColorError| e.to_string())?;

                message_sender.command(Command::Unset { pos, color });
            },
            "play" => {
                let pos: Pos = args.get(1).ok_or("position not provided.")?
                    .parse()
                    .map_err(|e: PosError| e.to_string())?;

                message_sender.command(Command::Play(pos.into()));
            },
            "undo" => {
                message_sender.command(Command::Undo);
            },
            "gen" => {
                message_sender.launch(SearchObjective::Best, false, false);
            },
            "igen" => {
                message_sender.launch(SearchObjective::Best, true, true);
            }
            "zero" => {
                message_sender.launch(SearchObjective::Zeroing, true, true);
            },
            "quit" => {
                std::process::exit(0);
            },
            &_ => return Err("unknown command.".to_string()),
        }
    }

    Ok(())
}
