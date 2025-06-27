#[cfg(test)]
mod test_search {
    use mintaka::config::Config;
    use mintaka::game_agent::GameAgent;
    use mintaka::protocol::command::Command;
    use mintaka::protocol::message::{Message, ResponseSender};
    use mintaka::protocol::response;
    use mintaka::protocol::response::Response;
    use rusty_renju::board::Board;
    use rusty_renju::history::History;
    use rusty_renju::notation::color::Color;
    use rusty_renju::utils::byte_size::ByteSize;
    use std::sync::atomic::AtomicBool;
    use std::sync::{mpsc, Arc};
    use std::time::Duration;

    fn search(board: Board) -> response::GameResult {
        let history = History::try_from(&board).unwrap();

        let config = Config::default();
        let aborted = Arc::new(AtomicBool::new(false));

        let (response_sender, message_receiver) = {
            let (message_sender, message_receiver) = mpsc::channel();
            (ResponseSender::new(message_sender), message_receiver)
        };

        let mut game_agent = GameAgent::new(config);

        game_agent.commands(&response_sender, vec![
            Command::Load(Box::new((board, history))),
            Command::MaxMemory(ByteSize::from_mib(128)),
            Command::TotalTime(Duration::ZERO),
            Command::IncrementTime(Duration::from_secs(1)),
            Command::TurnTime(Duration::from_secs(1)),
        ]).unwrap();

        game_agent = game_agent.launch(response_sender.clone(), aborted.clone());

        while let Ok(response) = message_receiver.try_recv() {
            match response {
                Message::Response(Response::Begins { workers, running_time, tt_size }) => {
                    println!("begins: workers={workers}, tt-size={tt_size}");
                },
                Message::Response(Response::BestMove { best_move, score, total_nodes_in_1k, time_elapsed }) => {
                    println!("solution: pos={best_move}, score={score}, nodes={total_nodes_in_1k}, elapsed={:?}", time_elapsed);
                    game_agent.command(&response_sender, Command::Play(best_move.into())).unwrap();
                    game_agent = game_agent.launch(response_sender.clone(), aborted.clone());
                },
                Message::Response(Response::Finished(result)) => {
                    println!("finished: result={:?}", result);
                    return result;
                }
                _ => {}
            }
        }

        unreachable!()
    }

    fn empty_position() {
        let board = Board::default();
        let result = search(board);
        assert_eq!(result, response::GameResult::Win(Color::Black));
    }

}
