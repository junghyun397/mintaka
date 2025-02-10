use crate::config::Config;
use crate::memo::transposition_table::TranspositionTable;
use crate::protocol::game_manager::GameManager;
use crate::search;
use crate::thread_data::ThreadData;
use rusty_renju::board::Board;
use std::sync::atomic::{AtomicBool, AtomicUsize};

pub fn launch(manager: impl GameManager, default_config: Config) {
    let tt = TranspositionTable::new_with_size(128 * 1024);
    let global_counter_in_1k = AtomicUsize::new(0);
    let global_aborted = AtomicBool::new(false);

    let mut td = ThreadData::new(&manager, default_config, &tt, &global_aborted, &global_counter_in_1k);

    let mut board = Board::default();

    let (best_move, score) = search::iterative_deepening(&mut td, &mut board);
}
