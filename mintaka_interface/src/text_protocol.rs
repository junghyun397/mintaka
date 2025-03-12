use mintaka::config::Config;
use mintaka::game_agent::GameAgent;
use mintaka::protocol::command::Command;
use mintaka::protocol::response::Response;

fn main() -> Result<(), &'static str> {
    let mut game_agent = GameAgent::new(Config::default());

    loop {
        match "d" {
            "parse-board" => {
                todo!()
            }
            "show-board" => {
                println!("board: total {} moves, {}'s turn.\n{}", 1, game_agent.state.board.player_color, game_agent.state.board);
            }
            "clear-board" => {
                todo!()
            }
            "set" => {
                todo!()
            }
            "unset" => {
                todo!()
            }
            "undo" => {
                todo!()
            }
            "batch-set" => {
                todo!()
            }
            "switch" => {
                todo!()
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
                                todo!()
                            }
                            Response::Pv(pos, pv) => {
                                println!("pv: pos={}, pv={}", pos, pv);
                            }
                            Response::BestMove(pos, score) => {
                                println!("solution: pos={}, score={}", pos, score);
                            },
                        }
                    }
                });
            }
            "status" => {
                todo!()
            }
            "abort" => {
                game_agent.command(Command::Abort);
            }
            "quite" => {
                return Ok(());
            }
            &_ => {
                println!("error: unknown command.");
            }
        }
    };
}
