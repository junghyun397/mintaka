use mintaka::config::Config;
use mintaka::game_agent::GameAgent;
use mintaka::protocol::command::Command;
use mintaka::protocol::response::Response;

fn main() {
    let mut game_agent = GameAgent::new(Config::default());

    loop {
        match "d" {
            "parse-board" => {
            }
            "show-board" => {
            }
            "clear-board" => {
            }
            "set" => {
            }
            "unset" => {
            }
            "batch-set" => {
            }
            "switch" => {
            }
            "gen" => {
                let channel = game_agent.launch();

                std::thread::spawn(move || {
                    for response in channel {
                        match response {
                            Response::Info(message) => {
                                println!("info: {}", message);
                            },
                            Response::Warning(message) => {
                                println!("warning: {}", message);
                            }
                            Response::Error(message) => {
                                println!("error: {}", message);
                            },
                            Response::Status(status) => {
                                println!("status: {}", 0);
                            }
                            Response::BestMove(pos, score) => {
                                println!("solution: {}, {}", pos, score);
                            },
                        }
                    }
                });
            }
            "status" => {
            }
            "abort" => {
                game_agent.command(Command::Abort);
            }
            "quite" => {
                break;
            }
            &_ => {
                println!("error: unknown command.");
            }
        }
    }
}
