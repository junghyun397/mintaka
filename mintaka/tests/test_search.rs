#[cfg(test)]
mod test_search {
    use indoc::indoc;
    use mintaka::config::Config;
    use mintaka::game_agent::GameAgent;
    use mintaka::game_state::GameState;
    use mintaka::protocol::response::NullResponseSender;
    use rusty_renju::board;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    macro_rules! test_search {
        ($source:expr) => {{
            let mut agent = {
                let mut config = Config::default();
                config.max_nodes_in_1k = Some(100);
                config.max_depth = 5;

                let state: GameState = $source.into();

                GameAgent::from_state(config, state)
            };

            let best_move = agent.launch(NullResponseSender, Arc::new(AtomicBool::new(false)));

            println!("{:?}", best_move);

            best_move
        }};
    }

    #[test]
    fn empty_position() {
        let source = board!(indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . . . . . . . . . . . . . 11
        10 . . . . . . . . . X . . . . . 10
         9 . . . . . . . O . O . . . . . 9
         8 . . . . . . . X X X O . . . . 8
         7 . . . . . . X O X . . . . . . 7
         6 . . . . . . . . O . . . . . . 6
         5 . . . . . . . . . . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O
       "});

        let best_move = test_search!(source);
    }
}