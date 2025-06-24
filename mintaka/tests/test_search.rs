#[cfg(test)]
mod test_search {
    use mintaka::config::Config;
    use mintaka::game_agent::GameAgent;
    use mintaka::protocol::command::Command;
    use mintaka::protocol::message::{Message, ResponseSender};
    use mintaka::protocol::response::Response;
    use std::sync::atomic::AtomicBool;
    use std::sync::{mpsc, Arc};
    use std::time::Duration;

    macro_rules! search {
        ($board:expr) => {{
            let mut board = $board;
            let config = Config::default();

            let launched = AtomicBool::new(false);
            let aborted = AtomicBool::new(false);

            let mut game_agent = GameAgent::new(config, aborted.clone());
        }};
    }

    #[test]
    fn empty_position() {
        let config = Config::default();
        let aborted = Arc::new(AtomicBool::new(false));

        let (response_sender, message_receiver) = {
            let (message_sender, message_receiver) = mpsc::channel();
            (ResponseSender::new(message_sender), message_receiver)
        };

        let mut game_agent = GameAgent::new(config, aborted.clone());

        game_agent.commands(vec![
            Command::TotalTime(Duration::ZERO),
            Command::IncrementTime(Duration::from_secs(1)),
            Command::TurnTime(Duration::from_secs(1)),
        ]).unwrap();

        game_agent.launch(response_sender.clone());

        while let Ok(response) = message_receiver.try_recv() {
            match response {
                Message::Response(Response::Begins { workers, running_time, tt_size }) => {
                    println!("begins: workers={workers}, tt-size={tt_size}");
                }
                Message::Response(Response::BestMove { best_move, score, total_nodes_in_1k, time_elapsed }) => {
                    println!("solution: pos={best_move}, score={score}, nodes={total_nodes_in_1k}, elapsed={:?}", time_elapsed);
                    game_agent.command(Command::Play(best_move.into())).unwrap();
                    game_agent.launch(response_sender.clone());
                }
                _ => {}
            }
        }
    }

}
