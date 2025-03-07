use crate::config::Config;
use crate::game_state::GameState;
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TranspositionTable;
use crate::protocol::game_manager::GameManager;
use crate::protocol::response::Response;
use crate::search;
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use rusty_renju::board::Board;
use rusty_renju::memo::abstract_transposition_table::AbstractTranspositionTable;
use rusty_renju::notation::pos::Pos;
use std::sync::atomic::{AtomicBool, AtomicUsize};

pub fn eager_launch(
    manager: &(impl GameManager + Sync),
    config: Config,
    board: Board,
) {
    let tt = TranspositionTable::new_with_size(1024 * 256);
    let mut ht = HistoryTable {};

    launch(
        manager,
        config,
        GameState {
            board,
            recent_move: Pos::INVALID,
            movegen_window: Default::default(),
        },
        &tt,
        &mut ht,
    );
}

pub fn launch(
    manager: &(impl GameManager + Sync),
    config: Config,
    mut state: GameState,
    tt: &TranspositionTable,
    ht: &mut HistoryTable,
) {
    let global_counter_in_1k = AtomicUsize::new(0);
    let global_aborted = AtomicBool::new(false);

    let mut td = ThreadData::new(
        ThreadType::Main, 0,
        config,
        tt.view(),
        ht.clone(),
        &global_aborted, &global_counter_in_1k,
    );

    std::thread::scope(|s| {
        s.spawn(|| {
            let (best_move, score) = search::iterative_deepening::<{ ThreadType::Main }>(&mut td, &mut state.clone());
            manager.response(Response::BestMove(best_move, score));
        });

        for tid in 1 .. config.workers.get() {
            let mut worker_td = ThreadData::new(
                ThreadType::Worker, tid,
                config,
                tt.view(),
                ht.clone(),
                &global_aborted, &global_counter_in_1k
            );

            s.spawn(move || {
                search::iterative_deepening::<{ ThreadType::Worker }>(&mut worker_td, &mut state.clone());
            });
        }
    });

    *ht = td.ht;
    tt.increase_age();
}
