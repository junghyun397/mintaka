#[cfg(test)]
mod test_result {
    use mintaka::board::Board;
    use mintaka::notation::color::Color;
    use std::str::FromStr;
    use indoc::indoc;
    use mintaka::notation::pos::Pos;

    #[test]
    fn five_in_a_row() {
        let case = indoc!{"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . . . . . . . . . . . . . 11
        10 . . . . . . . . . . . . . . . 10
         9 . . . . . . . . . . . . . . . 9
         8 . . . . . . . . . . . . . . . 8
         7 . . . . . . . . . . . . . . . 7
         6 . . . . . . . . . . . . . . . 6
         5 . . . . . . . . . . . . . X . 5
         4 . . . . . . . . . . . . X . . 4
         3 . . . . . . . . . . . X . . . 3
         2 . . . . . . . . . . X . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        let board = Board::from_str(case).unwrap()
            .set(Pos::from_str("o6").unwrap());

        assert_eq!(board.winner, Some(Color::Black));
    }

    #[test]
    fn full() {
        todo!()
    }

}
