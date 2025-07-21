use mintaka::config::Config;
use mintaka::game_agent::{ComputingResource, GameAgent, GameError};
use mintaka::protocol::command::Command;
use mintaka::protocol::message::{CommandSender, Message, MessageSender, StatusCommand};
use mintaka::protocol::response::{MpscResponseSender, Response};
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::notation::color::UnknownColorError;
use rusty_renju::notation::pos::{MaybePos, Pos, PosError};
use rusty_renju::utils::byte_size::ByteSize;
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

fn main() -> Result<(), GameError> {
    let launched = Arc::new(AtomicBool::new(false));
    let aborted = Arc::new(AtomicBool::new(false));

    let mut game_agent = GameAgent::new(Config::default());

    let (command_sender, message_sender, message_receiver) = {
        let (message_sender, message_receiver) = mpsc::channel();
        (CommandSender::new(message_sender.clone()), MessageSender::new(message_sender), message_receiver)
    };

    spawn_command_listener(launched.clone(), aborted.clone(), command_sender);

    for message in message_receiver {
        match message {
            Message::Command(command) => {
                let result = game_agent.command(&message_sender, command);

                if let Err(err) = result {
                    println!("error: {err}");
                }
            },
            Message::Status(command) => match command {
                StatusCommand::Version =>
                    println!("version: renju=rusty-renju-{}, engine=mintaka-{}, model=unknown", rusty_renju::VERSION, mintaka::VERSION),
                StatusCommand::Board =>
                    println!("{}", game_agent.state.board),
                StatusCommand::History =>
                    println!("history: {}", game_agent.state.history),
            },
            Message::Finished(result) => {
                println!("finished: {result}")
            }
            Message::Launch => {
                let (response_sender, response_receiver) = {
                    let (response_sender, response_receiver) = mpsc::channel();
                    (MpscResponseSender::new(response_sender), response_receiver)
                };

                std::thread::spawn(move || {
                    for response in response_receiver {
                        match response {
                            Response::Begins(ComputingResource { workers, time, nodes_in_1k, tt_size }) =>
                                println!("begins: workers={workers}, running-time={time:?}, nodes={nodes_in_1k}, tt-size={tt_size}"),
                            Response::Status { eval, total_nodes_in_1k, best_moves, hash_usage } =>
                                println!(
                                    "status: eval={eval}, \
                                    total_nodes_in_1k={total_nodes_in_1k}, \
                                    best_moves={best_moves:?}, \
                                    hash_usage={hash_usage}"
                                ),
                            Response::Pv(pvs) =>
                                println!("pvs={pvs:?}"),
                            Response::Finished => break
                        }
                    }
                });

                launched.store(true, Ordering::Relaxed);

                let resource = game_agent.next_computing_resource();

                let best_move = game_agent.launch(resource, response_sender.clone(), aborted.clone());

                launched.store(false, Ordering::Relaxed);

                println!(
                    "solution: pos={}, score={}, nodes={}k, elapsed={:?}",
                    best_move.pos, best_move.score, best_move.total_nodes_in_1k, best_move.time_elapsed
                );

                game_agent.command(&message_sender, Command::Play(best_move.pos))?;
                game_agent.command(&message_sender, Command::ConsumeTime(best_move.time_elapsed))?;
            },
        }
    }

    Ok(())
}

fn spawn_command_listener(launched: Arc<AtomicBool>, abort: Arc<AtomicBool>, command_sender: CommandSender) {
    std::thread::spawn(move || {
        let mut buf = String::new();

        loop {
            buf.clear();
            std::io::stdin().read_line(&mut buf).unwrap();
            let args = buf.trim().split(' ').collect::<Vec<&str>>();

            if args.len() == 0 {
                continue;
            }

            let result = handle_command(&launched, &abort, &command_sender, &buf, args);

            if let Err(err) = result {
                println!("error: {err}");
            }
        }
    });
}

fn handle_command(
    launched: &Arc<AtomicBool>, abort: &Arc<AtomicBool>, command_sender: &CommandSender, buf: &str, args: Vec<&str>
) -> Result<(), String> {
    if launched.load(Ordering::Relaxed) {
        match args[0] {
            "abort" => {
                abort.store(true, Ordering::Relaxed);
            },
            "quite" => std::process::exit(0),
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

                                command_sender.command(
                                    Command::Workers(NonZeroU32::new(cores).unwrap())
                                );
                            },
                            &_ => {
                                let workers = args.get(2)
                                    .ok_or("workers not provided.")?
                                    .parse::<u32>()
                                    .map_err(|_| "invalid workers number.")?;

                                command_sender.command(
                                    Command::Workers(NonZeroU32::new(workers).unwrap())
                                );
                            }
                        }
                    },
                    "memory" => {
                        let memory_size_in_kib = args.get(2)
                            .ok_or("memory not provided.")?
                            .parse::<usize>()
                            .map_err(|_| "invalid memory size.")?;

                        command_sender.command(Command::MaxMemory(ByteSize::from_kib(memory_size_in_kib)));
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
                            "match" => {
                                command_sender.command(
                                    Command::TotalTime(parse_time_in_milliseconds(&args)?)
                                );
                            },
                            "turn" => {
                                command_sender.command(
                                    Command::TurnTime(parse_time_in_milliseconds(&args)?)
                                );
                            },
                            "increment" => {
                                command_sender.command(
                                    Command::IncrementTime(parse_time_in_milliseconds(&args)?)
                                );
                            }
                            &_ => return Err("unknown time type.".to_string())
                        }
                    },
                    "nodes" => {
                        let nodes = args.get(1)
                            .ok_or("nodes not provided.")?
                            .parse::<usize>()
                            .map_err(|_| "invalid nodes number.")?;

                        command_sender.command(Command::MaxNodes { in_1k: nodes });
                    },
                    &_ => return Err("unknown limit type.".to_string()),
                }
            }
            "parse" => {
                match *args.get(1)
                    .ok_or("data type not provided.")?
                {
                    "board" => {
                        command_sender.command(Command::Load(
                            Box::new((buf.parse()?, History::default()))
                        ));
                    },
                    "history" => {
                        let history: History = args.get(2)
                            .ok_or("history not provided.")?
                            .parse()?;

                        let mut board = Board::default();

                        board.batch_set_mut(
                            &history.iter()
                                .copied()
                                .map(MaybePos::unwrap)
                                .collect::<Vec<Pos>>()
                        );

                        command_sender.command(Command::Load(
                            Box::new((board, history))
                        ))
                    },
                    &_ => return Err("unknown data type.".to_string()),
                }
            },
            "clear" => {
                command_sender.command(Command::Load(
                    Box::new((Board::default(), History::default()))
                ));
            },
            "b" | "board" => {
                command_sender.status(StatusCommand::Board);
            },
            "history" => {
                command_sender.status(StatusCommand::History);
            },
            "version" => {
                command_sender.status(StatusCommand::Version);
            },
            "set" => {
                let pos = args.get(1).ok_or("position not provided.")?.parse()
                    .map_err(|e: PosError| e.to_string())?;
                let color = args.get(2).ok_or("color not provided.")?.parse()
                    .map_err(|e: UnknownColorError| e.to_string())?;

                command_sender.command(Command::Set { pos, color });
            },
            "unset" => {
                let pos = args.get(1).ok_or("position not provided.")?.parse()
                    .map_err(|e: PosError| e.to_string())?;
                let color = args.get(2).ok_or("color not provided.")?.parse()
                    .map_err(|e: UnknownColorError| e.to_string())?;

                command_sender.command(Command::Unset { pos, color });
            },
            "p" | "play" => {
                let pos: Pos = args.get(1).ok_or("position not provided.")?.parse()
                    .map_err(|e: PosError| e.to_string())?;

                command_sender.command(Command::Play(pos.into()));
            },
            "u" | "undo" => {
                command_sender.command(Command::Undo);
            },
            "g" | "gen" => {
                command_sender.launch();
            },
            &_ => return Err("unknown command.".to_string()),
        }
    }

    Ok(())
}
