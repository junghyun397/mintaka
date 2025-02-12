use crate::config::Config;
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

pub fn launch(
    manager: &(impl GameManager + Sync),
    config: Config,
    threads: usize,
    search_limit: SearchLimit,
    tt: &TranspositionTable,
    ht: &mut HistoryTable,
) {
    let global_counter_in_1k = AtomicUsize::new(0);
    let global_aborted = AtomicBool::new(false);

    let mut td = ThreadData::new(
        0, config, search_limit,
        &tt, ht.clone(),
        &global_aborted, &global_counter_in_1k
    );

    let mut board = Board::default();

    std::thread::scope(|s| {
        s.spawn(|| {
            let (best_move, score) =
                search::iterative_deepening::<{ ThreadType::Main }>(&mut td, &mut board.clone());
            manager.response(Response::BestMove(best_move, score));
        });

        for tid in 1 .. threads {
            let mut worker_td = ThreadData::new(
                tid, config, search_limit,
                &tt, ht.clone(),
                &global_aborted, &global_counter_in_1k
            );

            s.spawn(move ||
                search::iterative_deepening::<{ ThreadType::Worker }>(&mut worker_td, &mut board.clone())
            );
        }
    });

    *ht = td.ht;
    tt.increase_age();
}
