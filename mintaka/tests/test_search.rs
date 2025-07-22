#[cfg(test)]
mod test_search {
    use mintaka::config::Config;
    use mintaka::game_agent::GameAgent;
    use mintaka::protocol::command::Command;
    use mintaka::protocol::message;
    use mintaka::protocol::message::{CommandSender, Message, MessageSender};
    use mintaka::protocol::response::{NullResponseSender, ResponseSender};
    use rusty_renju::board::Board;
    use rusty_renju::history::History;
    use rusty_renju::notation::color::Color;
    use rusty_renju::utils::byte_size::ByteSize;
    use std::sync::atomic::AtomicBool;
    use std::sync::{mpsc, Arc};
    use std::time::Duration;

    fn search(board: Board) -> message::GameResult {
        let history = History::try_from(&board).unwrap();

        let config = Config::default();
        let aborted = Arc::new(AtomicBool::new(false));

        let (command_sender, message_sender, message_receiver) = {
            let (message_sender, message_receiver) = mpsc::channel();
            (CommandSender::new(message_sender.clone()), MessageSender::new(message_sender), message_receiver)
        };

        let mut game_agent = GameAgent::new(config);

        game_agent.commands(&message_sender, vec![
            Command::Load(Box::new((board, history))),
            Command::MaxMemory(ByteSize::from_mib(128)),
            Command::IncrementTime(Duration::ZERO),
            Command::TurnTime(Duration::from_secs(1)),
        ]).unwrap();

        for message in message_receiver {
            match message {
                Message::Launch => {
                    let best_move = game_agent.launch(NullResponseSender, aborted.clone());

                    println!(
                        "solution: pos={}, score={}, nodes={}k, elapsed={:?}",
                        best_move.pos, best_move.score, best_move.total_nodes_in_1k, best_move.time_elapsed
                    );

                    game_agent.command(&message_sender, Command::Play(best_move.pos)).unwrap();

                    command_sender.launch();
                },
                Message::Finished(result) => return result,
                _ => {}
            }
        }

        unreachable!()
    }

    fn empty_position() {
        let board = Board::default();
        let result = search(board);
        assert_eq!(result, message::GameResult::Win(Color::Black));
    }

}
