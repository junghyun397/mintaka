#[cfg(test)]
mod test_board {
    use mintaka::board::Board;
    use mintaka::notation::pos::Pos;
    use std::str::FromStr;

    #[test]
    fn test_play() {
        let board = Board::default();

        let board = board
            .set(Pos::from_str("h8").unwrap())
            .set(Pos::from_str("g7").unwrap())
            .set(Pos::from_str("g9").unwrap());
        println!("{}", board.render_debug_board())
    }

    #[test]
    fn test_play_mut() {
        todo!()
    }

}
