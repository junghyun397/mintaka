mod template {
    use mintaka::config::Config;
    use mintaka::eval::evaluator::{ActiveEvaluator, Evaluator};
    use mintaka::game_agent::GameAgent;
    use mintaka::memo::history_table::HistoryTable;
    use mintaka::memo::transposition_table::TranspositionTable;
    use mintaka::protocol::response::NullResponseSender;
    use mintaka::thread_data::ThreadData;
    use mintaka::thread_type::WorkerThread;
    use rusty_renju::board::Board;
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    use std::sync::Arc;

    fn td() {
        let config = Config::default();
        let source = Board::default();

        let state = source.into();

        let evaluator = ActiveEvaluator::from_state(&state);

        let tt = TranspositionTable::new_with_size(config.tt_size);
        let ht = HistoryTable {};

        let global_counter_in_1k = AtomicUsize::new(0);
        let aborted = AtomicBool::new(false);

        let mut td = ThreadData::new(
            WorkerThread,
            0,
            config,
            evaluator,
            tt.view(),
            ht,
            &aborted,
            &global_counter_in_1k
        );
    }

    fn agent() {
        let mut agent = {
            let mut config = Config::default();
            config.max_nodes_in_1k = Some(1000);

            let source = Board::default();

            let state = source.into();

            GameAgent::from_state(config, state)
        };

        let best_move = agent.launch(NullResponseSender, Arc::new(AtomicBool::new(false)));

        println!("{:?}", best_move);
    }
}
