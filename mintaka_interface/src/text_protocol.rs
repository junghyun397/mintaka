use mintaka::config::Config;
use mintaka::game_agent::GameAgent;
use mintaka::protocol::command::Command;
use mintaka::protocol::message::{CommandSender, Message, ResponseSender};
use mintaka::protocol::response::Response;
use rusty_renju::history::Action;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::Pos;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

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

            if launched.load(Ordering::Relaxed) {
                match args[0] {
                    "abort" => command_sender.abort(),
                    "quite" => command_sender.quit(),
                    &_ => println!("error: unknown command.")
                }
            } else {
                match args[0] {
                    "set" => {
                        let pos = Pos::from_str(args[1]).unwrap();

                        command_sender.send(Command::Set { pos, color: Color::Black });
                    },
                    "unset" => {
                        let pos = Pos::from_str(args[1]).unwrap();

                        command_sender.send(Command::Unset { pos, color: Color::White });
                    },
                    "play" => {
                        let pos = Pos::from_str(args[1]).unwrap();

                        command_sender.send(Command::Play(Action::Move(pos)));
                    },
                    "undo" => {
                        command_sender.send(Command::Undo);
                    },
                    "switch" => {
                        command_sender.send(Command::Switch);
                    },
                    "gen" => {
                        command_sender.launch();
                    },
                    &_ => println!("error: unknown command."),
                }
            }
        }
    });
}

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
                game_agent.command(command);
            },
            Message::Response(response) => {
                match response {
                    Response::Info(message) =>
                        println!("info: {}", message),
                    Response::Warning(message) =>
                        println!("warning: {}", message),
                    Response::Error(message) =>
                        println!("error: {}", message),
                    Response::Status { total_nodes_in_1k, best_moves, hash_usage } => {
                        println!(
                            "status: total_nodes_in_1k={total_nodes_in_1k}, \
                            best_moves={best_moves:?}, \
                            hash_usage={hash_usage}"
                        );
                    }
                    Response::Pv(pos, pv) =>
                        println!("pv: pos={pos}, pv={pv}"),
                    Response::BestMove(pos, score) =>
                        println!("solution: pos={pos}, score={score}"),
                }
            },
            Message::Launch => {
                launched.store(true, Ordering::Relaxed);
                game_agent.launch(response_sender.clone());
            },
            Message::Abort => {
                launched.store(false, Ordering::Relaxed);
            },
            Message::Quit => {
                break;
            },
        }
    }

    Ok(())
}
