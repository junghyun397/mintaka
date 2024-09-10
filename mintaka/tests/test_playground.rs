#[cfg(test)]
mod playground {
    use mintaka::board::Board;
    use mintaka::notation::pos::Pos;

    #[test]
    fn playground() {
        let board = Board::default();

        println!("{:?}", board.slices.ascending_slices.iter().map(|x| x.start_pos).collect::<Box<[Pos]>>())
    }

}
