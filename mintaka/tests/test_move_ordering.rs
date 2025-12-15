#[cfg(test)]
mod test_movegen {
    use mintaka::config::{Config, SearchObjective};
    use mintaka::eval::evaluator::{ActiveEvaluator, Evaluator};
    use mintaka::memo::history_table::HistoryTable;
    use mintaka::memo::transposition_table::TranspositionTable;
    use mintaka::movegen::move_list::MoveEntry;
    use mintaka::movegen::move_picker::MovePicker;
    use mintaka::thread_data::ThreadData;
    use mintaka::thread_type::WorkerThread;
    use rusty_renju::history::History;
    use rusty_renju::notation::pos;
    use rusty_renju::notation::pos::MaybePos;
    use std::sync::atomic::{AtomicBool, AtomicU64};

    macro_rules! test_move_ordering {
        ($history:literal) => {{
            let config = Config::default();
            let history: History = $history.parse().unwrap();

            let state = history.into();

            let mut evaluator = ActiveEvaluator::from_state(&state);

            let tt = TranspositionTable::new_with_size(config.tt_size);
            let ht = HistoryTable::EMPTY;

            let global_counter_in_1k = AtomicU64::new(0);
            let aborted = AtomicBool::new(false);

            let mut td = ThreadData::new(
                WorkerThread, 0,
                SearchObjective::Best,
                config,
                evaluator,
                tt.view(),
                ht,
                &aborted,
                &global_counter_in_1k
            );

            let mut move_picker = MovePicker::init_new(MaybePos::NONE, [MaybePos::NONE; 2], state.board.is_forced_defense());

            let mut heatmap = [f32::NAN; pos::BOARD_SIZE];
            while let Some(MoveEntry { pos, move_score: score }) = move_picker.next(&mut td, &state) {
                heatmap[pos.idx_usize()] = score as f32;

                print!("{:?}, ", (pos, score));
            }

            println!("\n{}", state.board.to_string_with_heatmap(heatmap, true));
        }};
    }

    #[test]
    fn move_ordering() {
        test_move_ordering!("h8h7h6k7j8");
    }

}
