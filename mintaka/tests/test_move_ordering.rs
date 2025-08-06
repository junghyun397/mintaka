#[cfg(test)]
mod test_movegen {
    use mintaka::game_state::GameState;
    use mintaka::movegen::move_picker::MovePicker;
    use rusty_renju::history::History;
    use rusty_renju::notation::pos;
    use rusty_renju::notation::pos::MaybePos;

    macro_rules! test_move_ordering {
        ($history:literal) => {{
            let history: History = $history.parse().unwrap();
            let state = GameState::from_board_and_history(history.into(), history);

            let mut move_picker = MovePicker::new(MaybePos::NONE, [MaybePos::NONE; 2]);

            let mut heatmap = [f64::NAN; pos::BOARD_SIZE];
            while let Some((pos, score)) = move_picker.next(&state) {
                heatmap[pos.idx_usize()] = score as f64;

                print!("{:?}, ", (pos, score));
            }

            println!("\n{}", state.board.to_string_with_heatmap(heatmap, true));
        }};
    }

    #[test]
    fn move_ordering() {
        test_move_ordering!("h8h7h6");
    }

}
