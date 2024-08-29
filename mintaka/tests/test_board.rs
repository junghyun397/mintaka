#[cfg(test)]
mod test_board {
    use std::str::FromStr;
    use mintaka::board::Board;
    use mintaka::notation::pos::Pos;

    #[test]
    fn test_play() {
        let mut board = Board::default();
        board.set_mut(Pos::from_str("h8").unwrap());
        board.set_mut(Pos::from_str("h8").unwrap());
        println!("{}", board)
    }

    #[test]
    fn test_play_mut() {
        todo!()
    }

}
