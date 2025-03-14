use mintaka::config::Config;
use mintaka::game_agent::GameAgent;
use mintaka::protocol::response::Response;
use rusty_renju::notation::pos::Pos;
use std::str::FromStr;

fn main() -> Result<(), &'static str> {
    let mut game_agent = GameAgent::new(Config::default());

    loop {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).map_err(|_| "failed to stdio")?;
        let args = buf.trim().split(' ').collect::<Vec<&str>>();

        if args.len() == 0 {
            continue;
        }

        let parameters = &args[1..];

        match args[0] {
            "parse-board" => {
            },
            "show-board" => {
                println!(
                    "board: total {} moves, {}'s turn.\n{}",
                    0,
                    game_agent.state.board.player_color,
                    game_agent.state.board
                );
            },
            "clear-board" => {
                todo!()
            },
            "set" => {
                let pos = Pos::from_str(parameters[0]).map_err(|_| "pos parsing failed.")?;
                todo!()
            },
            "unset" => {
                let pos = Pos::from_str(parameters[0]).map_err(|_| "pos parsing failed.")?;
                todo!()
            },
            "play" => {
                let pos = Pos::from_str(parameters[0]).map_err(|_| "pos parsing failed.")?;
                game_agent.play(pos);
            },
            "undo" => {
                game_agent.undo();
            },
            "batch-set" => {
                todo!()
            },
            "switch" => {
                todo!()
            },
            "gen" => {
                let channel = game_agent.launch(todo!());

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
                            Response::Status { nps, total_nodes_in_1k, hash_usage, best_moves } => {
                                todo!()
                            }
                            Response::Pv(pos, pv) => {
                                println!("pv: pos={}, pv={}", pos, pv);
                            }
                            Response::BestMove(pos, score) => {
                                game_agent.state.board.set_mut(pos);
                                println!("solution: pos={}, score={}", pos, score);
                            },
                        }
                    }
                });
            },
            "status" => {
                todo!()
            },
            "abort" => {
                todo!()
            },
            "reset" => {
                game_agent = GameAgent::new(Config::default());
            },
            "quite" => {
                return Ok(());
            },
            &_ => { println!("error: unknown command."); }
        }
    };
}
