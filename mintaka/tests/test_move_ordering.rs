#[cfg(test)]
mod test_movegen {
    use mintaka::game_state::GameState;
    use mintaka::movegen::move_picker::MovePicker;
    use rusty_renju::history::History;
    use rusty_renju::notation::pos::MaybePos;

    macro_rules! test_move_ordering {
        ($history:literal) => {{
            let history: History = $history.parse().unwrap();
            let state = GameState::from(history);

            let mut move_picker = MovePicker::new(MaybePos::NONE, [MaybePos::NONE; 2]);

            while let Some(tuple) = move_picker.next(&state) {
                print!("{tuple:?}, ");
            }
        }};
    }

    #[test]
    fn move_ordering() {
        test_move_ordering!("h8h7h6");
    }

}
