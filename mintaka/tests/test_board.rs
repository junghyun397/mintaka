#[cfg(test)]
mod test_board {
    use mintaka::board::Board;
    use mintaka::notation::pos::Pos;
    use std::str::FromStr;

    #[test]
    fn test_play() {
        let mut board = Board::default();
        board.set_mut(Pos::from_str("h8").unwrap());
        println!("{}", board.render_debug_board(false))
    }

    #[test]
    fn test_play_mut() {
        todo!()
    }

}
