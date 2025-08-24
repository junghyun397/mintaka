use mintaka::config::Config;
use mintaka::game_agent::{ComputingResource, GameAgent, GameError};
use mintaka::game_state::GameState;
use mintaka::protocol::command::Command;
use mintaka::protocol::message::{Message, MessageSender, StatusCommand};
use mintaka::protocol::response::{CallBackResponseSender, Response};
use mintaka_interface::preference::{Mode, Preference};
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::notation::color::UnknownColorError;
use rusty_renju::notation::pos::{Pos, PosError};
use rusty_renju::utils::byte_size::ByteSize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

fn main() -> Result<(), GameError> {
    let pref = Preference::parse();

    match pref.mode {
        Mode::TextProtocol => text_protocol(pref.default_config, pref.game_state.unwrap_or_default()),
        Mode::SelfPlay => self_play(pref.default_config, pref.game_state.unwrap()),
    }
}

fn text_protocol(config: Config, state: GameState) -> Result<(), GameError> {
    let launched = Arc::new(AtomicBool::new(false));
    let aborted = Arc::new(AtomicBool::new(false));

    let mut game_agent = GameAgent::from_state(config, state);

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
                launched.store(true, Ordering::Relaxed);

                let best_move = game_agent.launch(CallBackResponseSender::new(response_printer), aborted.clone());

                launched.store(false, Ordering::Relaxed);

                println!(
                    "solution: pos={}, score={}, depth={}, nodes={}k, elapsed={:?}",
                    best_move.pos, best_move.score, best_move.depth_reached, best_move.total_nodes_in_1k, best_move.time_elapsed
                );

                let result = game_agent.command(Command::Play(best_move.pos))?;
                message_sender.result(result);
            },
        }
    }

    Ok(())
}

fn response_printer(response: Response) {
    match response {
        Response::Begins(ComputingResource { workers, time, nodes_in_1k, tt_size }) =>
            println!("begins: workers={workers}, running-time={time:?}, nodes={nodes_in_1k:?}, tt-size={tt_size}"),
        Response::Status { score, pv, total_nodes_in_1k, depth, hash_usage } =>
            println!(
                "status: score={score}, \
                pv={pv:?}, \
                total_nodes_in_1k={total_nodes_in_1k}, \
                depth={depth}, \
                hash_usage={hash_usage}"
            ),
        _ => {}
    }

}

fn spawn_command_listener(launched: Arc<AtomicBool>, abort: Arc<AtomicBool>, message_sender: MessageSender) {
    std::thread::spawn(move || {
        let mut buf = String::new();

        loop {
            buf.clear();
            std::io::stdin().read_line(&mut buf).unwrap();
            let args = buf.trim().split(' ').collect::<Vec<&str>>();

            if args.len() == 0 {
                continue;
            }

            let result = handle_command(&launched, &abort, &message_sender, &buf, args);

            if let Err(err) = result {
                println!("error: {err}");
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
                            .parse::<usize>()
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
                            "match" => {
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
                            .parse::<usize>()
                            .map_err(|_| "invalid nodes number.")?;

                        message_sender.command(Command::MaxNodes { in_1k: nodes });
                    },
                    &_ => return Err("unknown limit type.".to_string()),
                }
            }
            "parse" => {
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
                        ))
                    },
                    &_ => return Err("unknown data type.".to_string()),
                }
            },
            "clear" => {
                message_sender.command(Command::Load(
                    Box::new((Board::default(), History::default()))
                ));
            },
            "b" | "board" => {
                message_sender.status(StatusCommand::Board);
            },
            "history" => {
                message_sender.status(StatusCommand::History);
            },
            "version" => {
                message_sender.status(StatusCommand::Version);
            },
            "set" => {
                let pos = args.get(1).ok_or("position not provided.")?.parse()
                    .map_err(|e: PosError| e.to_string())?;
                let color = args.get(2).ok_or("color not provided.")?.parse()
                    .map_err(|e: UnknownColorError| e.to_string())?;

                message_sender.command(Command::Set { pos, color });
            },
            "unset" => {
                let pos = args.get(1).ok_or("position not provided.")?.parse()
                    .map_err(|e: PosError| e.to_string())?;
                let color = args.get(2).ok_or("color not provided.")?.parse()
                    .map_err(|e: UnknownColorError| e.to_string())?;

                message_sender.command(Command::Unset { pos, color });
            },
            "p" | "play" => {
                let pos: Pos = args.get(1).ok_or("position not provided.")?.parse()
                    .map_err(|e: PosError| e.to_string())?;

                message_sender.command(Command::Play(pos.into()));
            },
            "u" | "undo" => {
                message_sender.command(Command::Undo);
            },
            "g" | "gen" => {
                message_sender.launch();
            },
            &_ => return Err("unknown command.".to_string()),
        }
    }

    Ok(())
}

fn self_play(config: Config, game_state: GameState) -> Result<(), GameError> {
    let aborted = Arc::new(AtomicBool::new(false));

    let (message_sender, message_receiver) = {
        let (tx, rx) = mpsc::channel();
        (MessageSender::new(tx), rx)
    };

    let mut game_agent = GameAgent::from_state(config, game_state);

    // game_agent.command(Command::Workers(num_cpus::get_physical() as u32))?;

    message_sender.launch();

    let mut game_result = None;
    for message in message_receiver {
        match message {
            Message::Launch => {
                let best_move = game_agent.launch(CallBackResponseSender::new(response_printer), aborted.clone());

                let result = game_agent.command(Command::Play(best_move.pos))?;
                message_sender.result(result);

                println!("{}",
                    game_agent.state.board.to_string_with_last_moves(game_agent.state.history.recent_action_pair())
                );

                println!(
                    "solution: pos={}, score={}, depth={}, nodes={}k, elapsed={:?}",
                    best_move.pos, best_move.score, best_move.depth_reached, best_move.total_nodes_in_1k, best_move.time_elapsed
                );

                message_sender.launch();
            },
            Message::Finished(result) => {
                game_result = Some(result);
                break;
            },
            _ => {}
        }
    }

    println!("{}", game_result.unwrap());
    println!("total {}k nodes", game_agent.overall_nodes_in_1k);

    Ok(())
}
