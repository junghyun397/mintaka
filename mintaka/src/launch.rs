use crate::config::Config;
use crate::eval::heuristic_evaluator::HeuristicEvaluator;
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TranspositionTable;
use crate::protocol::game_manager::GameManager;
use crate::protocol::response::Response;
use crate::search;
use crate::search_limit::SearchLimit;
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use rusty_renju::board::Board;
use std::sync::atomic::{AtomicBool, AtomicUsize};

pub fn eager_launch(
    manager: &(impl GameManager + Sync),
    search_limit: SearchLimit,
) {
    let tt = TranspositionTable::new_with_size(1024 * 16);
    let mut ht = HistoryTable {};

    launch(
        manager,
        Config::default(),
        1,
        search_limit,
        &HeuristicEvaluator::default(),
        &tt,
        &mut ht,
    );
}

pub fn launch(
    manager: &(impl GameManager + Sync),
    config: Config,
    threads: usize,
    search_limit: SearchLimit,
    evaluator: &HeuristicEvaluator,
    tt: &TranspositionTable,
    ht: &mut HistoryTable,
) {
    let global_counter_in_1k = AtomicUsize::new(0);
    let global_aborted = AtomicBool::new(false);

    let mut td = ThreadData::new(
        0, config, search_limit,
        &evaluator,
        &tt, ht.clone(),
        &global_aborted, &global_counter_in_1k
    );

    let mut board = Board::default();

    let mut handles = vec![];

    std::thread::scope(|s| {
        let main_handle = s.spawn(|| {
            let (best_move, score) =
                search::iterative_deepening::<{ ThreadType::Main }>(&mut td, &mut board.clone());
            manager.response(Response::BestMove(best_move, score));
        });

        handles.push(main_handle);

        for tid in 1 .. threads {
            let mut worker_td = ThreadData::new(
                tid, config, search_limit,
                &evaluator,
                &tt, ht.clone(),
                &global_aborted, &global_counter_in_1k
            );

            let worker_handle = s.spawn(move || {
                search::iterative_deepening::<{ ThreadType::Worker }>(&mut worker_td, &mut board.clone());
            });

            handles.push(worker_handle);
        }
    });

    for handle in handles {
        handle.join().unwrap();
    }

    *ht = td.ht;
    tt.increase_age();
}
