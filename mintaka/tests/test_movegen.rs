#[cfg(test)]
mod test_movegen {
    use mintaka::game_state::GameState;
    use mintaka::movegen::move_picker::MovePicker;
    use rusty_renju::history::History;
    use rusty_renju::notation::pos::MaybePos;
    use std::str::FromStr;

    macro_rules! movegen {
        ($board:expr) => {{
            let mut board = $board;
        }}
    }

    #[test]
    fn move_ordering() {
        let history = History::from_str("h8,i8,h6").unwrap();
        let state = GameState::from(history);

        let mut move_picker = MovePicker::new(MaybePos::NONE, [MaybePos::NONE; 2]);

        let mut acc = vec![];
        while let Some(tuple) = move_picker.next(&state) {
            acc.push(tuple);
        }

        println!("{}", state.board);
        println!("{acc:?}");
    }

}
