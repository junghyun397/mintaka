#[cfg(test)]
mod test_movegen {
    use mintaka::config::Config;
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
    use std::sync::atomic::{AtomicBool, AtomicUsize};

    macro_rules! test_move_ordering {
        ($history:literal) => {{
            let config = Config::default();
            let history: History = $history.parse().unwrap();

            let state = history.into();

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

            let mut move_picker = MovePicker::new(MaybePos::NONE, [MaybePos::NONE; 2]);

            let mut heatmap = [f32::NAN; pos::BOARD_SIZE];
            while let Some(MoveEntry { pos, score }) = move_picker.next(&td, &state) {
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
