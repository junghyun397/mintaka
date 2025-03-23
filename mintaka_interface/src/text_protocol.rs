use mintaka::config::Config;
use mintaka::game_agent::GameAgent;
use mintaka::protocol::command::Command;
use mintaka::protocol::message::{CommandSender, Message, ResponseSender, StatusCommand};
use mintaka::protocol::response::Response;
use rusty_renju::board::Board;
use rusty_renju::history::{Action, History};
use rusty_renju::notation::pos::Pos;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

fn main() -> Result<(), &'static str> {
    let mut game_agent = GameAgent::new(Config::default());

    let launched = Arc::new(AtomicBool::new(false));

    let (command_sender, response_sender, message_receiver) = {
        let (message_sender, message_receiver) = mpsc::channel();
        (CommandSender::new(message_sender.clone()), ResponseSender::new(message_sender), message_receiver)
    };

    spawn_command_listener(launched.clone(), command_sender);

    for message in message_receiver {
        match message {
            Message::Command(command) => {
                let result = game_agent.command(command);

                if let Err(err) = result {
                    println!("error: {}", err);
                }
            },
            Message::Response(response) => {
                match response {
                    Response::Info(message) =>
                        println!("info: {}", message),
                    Response::Warning(message) =>
                        println!("warning: {}", message),
                    Response::Error(message) =>
                        println!("error: {}", message),
                    Response::Status { eval, total_nodes_in_1k, best_moves, hash_usage } =>
                        println!(
                            "status: eval={eval}\
                            total_nodes_in_1k={total_nodes_in_1k}, \
                            best_moves={best_moves:?}, \
                            hash_usage={hash_usage}"
                        ),
                    Response::Pv(pvs) =>
                        println!("pvs={pvs:?}"),
                    Response::BestMove(pos, score) => {
                        launched.store(false, Ordering::Relaxed);

                        println!("solution: pos={pos}, score={score}");
                    }
                }
            },
            Message::Status(command) => match command {
                StatusCommand::Version =>
                    println!("version: engine=mintaka-{}, model=unknown", mintaka::VERSION),
                StatusCommand::Board =>
                    println!("{}", game_agent.state.board),
                StatusCommand::History =>
                    println!("history: {}", game_agent.history),
            },
            Message::Launch => {
                launched.store(true, Ordering::Relaxed);
                game_agent.launch(response_sender.clone());
            },
            Message::Abort => {
                launched.store(false, Ordering::Relaxed);
                game_agent.abort();
            },
            Message::Quit => {
                break;
            },
        }
    }

    Ok(())
}

fn spawn_command_listener(launched: Arc<AtomicBool>, command_sender: CommandSender) {
    std::thread::spawn(move || {
        let mut buf = String::new();

        loop {
            buf.clear();
            std::io::stdin().read_line(&mut buf).unwrap();
            let args = buf.trim().split(' ').collect::<Vec<&str>>();

            if args.len() == 0 {
                continue;
            }

            let result = handle_command(&launched, &command_sender, &buf, args);

            if let Err(err) = result {
                println!("error: {err}");
            }
        }
    });
}

fn handle_command(
    launched: &Arc<AtomicBool>, command_sender: &CommandSender, buf: &str, args: Vec<&str>
) -> Result<(), &'static str> {
    if launched.load(Ordering::Relaxed) {
        match args[0] {
            "abort" => command_sender.abort(),
            "quite" => command_sender.quit(),
            &_ => return Err("unknown command.")
        }
    } else {
        match args[0] {
            "config" => {
                match *args.get(1)
                    .ok_or("data type not provided.")?
                {
                    "time" => {
                        fn parse_time_in_milliseconds(args: &Vec<&str>) -> Result<Duration, &'static str> {
                            let time = args.get(1)
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
                            &_ => return Err("unknown time type.")
                        }
                    },
                    "workers" => {
                        match *args.get(2)
                            .ok_or("workers not provided.")?
                        {
                            "auto" => {
                                let cores = num_cpus::get_physical();

                                println!("info: workers={cores}");

                                command_sender.command(
                                    Command::Workers(NonZeroUsize::new(cores).unwrap())
                                );
                            },
                            &_ => {
                                let workers = args.get(2)
                                    .ok_or("workers not provided.")?
                                    .parse::<usize>()
                                    .map_err(|_| "invalid workers number.")?;

                                command_sender.command(
                                    Command::Workers(NonZeroUsize::new(workers).unwrap())
                                );
                            }
                        }
                    },
                    "memory" => {
                        let memory = args.get(2)
                            .ok_or("memory not provided.")?
                            .parse::<usize>()
                            .map_err(|_| "invalid memory size.")?;

                        command_sender.command(Command::MaxMemory { in_kib: memory });
                    },
                    &_ => return Err("data type not provided.")
                }
            },
            "parse" => {
                match *args.get(1)
                    .ok_or("data type not provided.")?
                {
                    "board" => {
                        command_sender.command(Command::Load(
                            Box::new(buf.parse()?), History::default()
                        ));
                    },
                    "history" => {
                        let history: History = args.get(2)
                            .ok_or("history not provided.")?
                            .parse()?;

                        let mut board = Board::default();

                        board.batch_set_mut(
                            &history.0.iter()
                                .map(Action::unwrap)
                                .collect::<Vec<Pos>>()
                        );

                        command_sender.command(Command::Load(
                            Box::new(board), history
                        ))
                    },
                    &_ => return Err("unknown data type."),
                }
            },
            "board" => {
                command_sender.status(StatusCommand::Board);
            },
            "history" => {
                command_sender.status(StatusCommand::History);
            },
            "version" => {
                command_sender.status(StatusCommand::Version);
            },
            "set" => {
                let pos = args.get(1).ok_or("pos not provided.")?.parse()?;
                let color = args.get(2).ok_or("color not provided.")?.parse()?;

                command_sender.command(Command::Set { pos, color });
            },
            "unset" => {
                let pos = args.get(1).ok_or("pos not provided.")?.parse()?;
                let color = args.get(2).ok_or("color not provided.")?.parse()?;

                command_sender.command(Command::Unset { pos, color });
            },
            "play" => {
                let pos = args.get(1).ok_or("pos not provided.")?.parse()?;

                command_sender.command(Command::Play(Action::Move(pos)));
            },
            "undo" => {
                command_sender.command(Command::Undo);
            },
            "gen" => {
                command_sender.launch();
            },
            &_ => return Err("unknown command."),
        }
    }

    Ok(())
}
