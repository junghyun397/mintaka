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

    fn empty_position() {
        let config = Config::default();
        let aborted = Arc::new(AtomicBool::new(false));

        let (response_sender, message_receiver) = {
            let (message_sender, message_receiver) = mpsc::channel();
            (ResponseSender::new(message_sender), message_receiver)
        };

        let mut game_agent = GameAgent::new(config, aborted.clone());

        game_agent.command(Command::TotalTime(Duration::ZERO)).unwrap();
        game_agent.command(Command::IncrementTime(Duration::from_secs(1))).unwrap();
        game_agent.launch(response_sender.clone());

        while let Ok(response) = message_receiver.try_recv() {
            if let Message::Response(Response::BestMove(pos, score)) = response {
                println!("solution: pos={pos}, score={score}");
                game_agent.command(Command::Play(pos.into())).unwrap();
                game_agent.launch(response_sender.clone());
            }
        }
    }

}
