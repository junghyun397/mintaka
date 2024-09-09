#[cfg(test)]
mod test_result {
    use mintaka::board::Board;
    use mintaka::notation::color::Color::White;
    use mintaka::utils::str_utils::trim_indent;
    use std::str::FromStr;

    #[test]
    fn five_in_a_row() {
        let origin = trim_indent("\
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . . . . . . . . . . . . . 11
        10 . . . . . . . . X . . . . . . 10
         9 . . . . . . . X . . . . . . . 9
         8 . . . . . . X . . . . . . . . 8
         7 . . . . . X . . . . . . . . . 7
         6 . . . . X . . . . . . . . . . 6
         5 . . . . . . . . . . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O");

        assert_eq!(Board::from_str(&origin).unwrap().winner, Some(White))
    }

    #[test]
    fn full() {
        todo!()
    }

    #[test]
    fn forbidden_move() {
        todo!()
    }

    #[test]
    fn only_forbidden_points_remaining() {
        todo!()
    }

}
