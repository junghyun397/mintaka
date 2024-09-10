#[cfg(test)]
mod test_forbid {
    use std::str::FromStr;
    use indoc::indoc;
    use mintaka::board::Board;

    #[test]
    fn basic_forbidden_moves() {
        let case = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . X . . . . . . . . . . 13
        12 . . X . X . . . . . . . . . . 12
        11 . X . . X . . . . . . . . . . 11
        10 X . . . O . . . . . . . . . . 10
         9 . . . . . . . . . . . . . . . 9
         8 . . . X X . X X X . . . . . . 8
         7 . . . . . . . . . . . . . . . 7
         6 . . . . . . . . . . . . . . . 6
         5 . . . . . . . . . . . . . . . 5
         4 . X . . . . . . . . . X X . . 4
         3 . X . X . . . . . . X . X . . 3
         2 . . . X . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        let expected = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . 4 . . . . . . . . . . 14
        13 . . . . X . . . . . . . . . . 13
        12 . . X . X . . . . . . . . . . 12
        11 . X . . X . . . . . . . . . . 11
        10 X . . . O . . . . . . . . . . 10
         9 . . . . . . . . . . . . . . . 9
         8 . . . X X 6 X X X . . . . . . 8
         7 . . . . . . . . . . . . . . . 7
         6 . . . . . . . . . . . . . . . 6
         5 . . . . . . . . . . . . 3 . . 5
         4 . X . . . . . . . . . X X . . 4
         3 . X . X . . . . . . X . X . . 3
         2 . . . X . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        assert_eq!(Board::from_str(case).unwrap().to_string(), expected);
    }

    #[test]
    fn basic_double_three() {
        todo!()
    }

    #[test]
    fn basic_double_four() {
        todo!()
    }

    #[test]
    fn overline() {
        todo!()
    }

    #[test]
    fn five_in_a_row() {
        todo!()
    }

}
