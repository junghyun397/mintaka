#[cfg(test)]
mod test_search {
    use indoc::indoc;
    use mintaka::config::{Config, SearchObjective};
    use mintaka::game_agent::GameAgent;
    use mintaka::game_state::GameState;
    use mintaka::protocol::response::NullResponseSender;
    use rusty_renju::board;
    use std::sync::atomic::{AtomicBool, AtomicU32};
    use std::sync::Arc;
    use std::time::Instant;
    use rusty_renju::notation::rule::RuleKind;

    macro_rules! test_search {
        ($source:expr) => {{
            let mut config = Config::default();
            config.workers = 8;
            config.max_nodes_in_1k = Some(100);

            let mut agent = {
                let state: GameState<{ RuleKind::Renju }> = $source.into();

                GameAgent::from_state(config, state)
            };

            let best_move = agent.launch::<Instant>(
                config,
                config.initial_timer,
                SearchObjective::Best,
                NullResponseSender,
                Arc::new(AtomicU32::new(0)),
                Arc::new(AtomicBool::new(false))
            );

            println!("{:?}", best_move);
        }};
    }

    #[test]
    fn basic_position() {
        let source = board!(indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . . . . . . . . . . . . . 11
        10 . . . . . . . . . . . . . . . 10
         9 . . . . . . . . . . . . . . . 9
         8 . . . . . . . X . . . . . . . 8
         7 . . . . . . . . . . . . . . . 7
         6 . . . . . . . . . . . . . . . 6
         5 . . . . . . . . . . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O
       "});

        test_search!(source);
    }
}