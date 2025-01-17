#[cfg(test)]
mod test_nested_forbid {
    use indoc::indoc;
    use rusty_renju::board::Board;
    use std::str::FromStr;

    #[test]
    fn single_nested_double_three() {
        let case = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . O . . . . . . . 12
        11 . . . . . . . X . . . . . . . 11
        10 . . . . . O . X . . . . . . . 10
         9 . . . . . X O X O . . . . . . 9
         8 . . . . . X . X . O . . . . . 8
         7 . . O X X X X O X . . . . . . 7
         6 . . . . . X . O . O . . . . . 6
         5 . . . . X O O . . . . . . . . 5
         4 . . . O . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        let expected = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . O . . . . . . . 12
        11 . . . . . . . X . . . . . . . 11
        10 . . . . . O . X . . . . . . . 10
         9 . . . . . X O X O . . . . . . 9
         8 . . . . 3 X . X . O . . . . . 8
         7 . . O X X X X O X . . . . . . 7
         6 . . . . 3 X . O . O . . . . . 6
         5 . . . . X O O . . . . . . . . 5
         4 . . . O . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        assert_eq!(Board::from_str(case).unwrap().to_string(), expected);
    }

    #[cfg(feature = "strict_renju")]
    #[test]
    fn double_nested_double_three() {
        let case = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . O . . O . . . . . . . . 11
        10 . . . . X . . X . O . . . . . 10
         9 . . . . O X O X X . . . . . . 9
         8 . . . . . . X X . . . . . . . 8
         7 . . . . . . O O X X . . . . . 7
         6 . . . . . X . . . . . . . . . 6
         5 . . . . O X . . . . . . . . . 5
         4 . . . . . O . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        let expected = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . O . . O . . . . . . . . 11
        10 . . . . X . . X . O . . . . . 10
         9 . . . . O X O X X . . . . . . 9
         8 . . . . . . X X 3 . . . . . . 8
         7 . . . . . . O O X X . . . . . 7
         6 . . . . . X . . . . . . . . . 6
         5 . . . . O X . . . . . . . . . 5
         4 . . . . . O . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        assert_eq!(Board::from_str(case).unwrap().to_string(), expected);

        let case = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . O . . O . . . . . . . . 11
        10 . . . . X . . X . O . . . . . 10
         9 . . . . O X O X X . . . . . . 9
         8 . . . . . . X X . . O . . . . 8
         7 . . . . . . O O X X . . . . . 7
         6 . . . . . X O . . . . . . . . 6
         5 . . . . O X . . . . . . . . . 5
         4 . . . . . O . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        let expected = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . O . . O . . . . . . . . 11
        10 . . . . X . 3 X 3 O . . . . . 10
         9 . . . . O X O X X . . . . . . 9
         8 . . . . . . X X 3 . O . . . . 8
         7 . . . . . . O O X X . . . . . 7
         6 . . . . . X O . . . . . . . . 6
         5 . . . . O X . . . . . . . . . 5
         4 . . . . . O . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        assert_eq!(Board::from_str(case).unwrap().to_string(), expected);
    }

    #[cfg(feature = "strict_renju")]
    #[test]
    fn pseudo_double_nested_double_three() {
        let case = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . . . X . . . O . . . . . 11
        10 . . . . . . . . . . X . . . . 10
         9 . . . . . . . . . . O X . . . 9
         8 . . . . . O X X X O X . . . . 8
         7 . . . . . X . . . . . . . . . 7
         6 . . . . . . . X . . . . . . . 6
         5 . . . . . . . . . X . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        let expected = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . . . X . . . O . . . . . 11
        10 . . . . . . . . . . X . . . . 10
         9 . . . . . . . 3 . . O X . . . 9
         8 . . . . . O X X X O X . . . . 8
         7 . . . . . X . 3 . . . . . . . 7
         6 . . . . . . . X 3 . . . . . . 6
         5 . . . . . . . . . X . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        assert_eq!(Board::from_str(case).unwrap().to_string(), expected);
    }

    #[cfg(feature = "strict_renju")]
    #[test]
    fn multiple_nested_double_three() {
        let case = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . X . . . . . . . 12
        11 . . . . . . . . . . . . X . . 11
        10 . . . . . O . . . . . . O X . 10
         9 . . O X . . X O X X X O X . . 9
         8 . . . X O O . X O . . . . . . 8
         7 . O . . X X . . . X . . . . . 7
         6 . . . . . . X . . . X X O . . 6
         5 . . . . O . . . . . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        let expected = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . X . . . . . . . 12
        11 . . . . . . . . . . . . X . . 11
        10 . . . . . O . . . 3 . . O X . 10
         9 . . O X . . X O X X X O X . . 9
         8 . . . X O O . X O 3 . . . . . 8
         7 . O . . X X 3 . . X 3 . . . . 7
         6 . . . . . . X . . . X X O . . 6
         5 . . . . O . . . . . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        assert_eq!(Board::from_str(case).unwrap().to_string(), expected);

        let case = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . X . . . . . . . 12
        11 . . . . . . . . . . . . X . X 11
        10 . . . . . O . . . . . . O X . 10
         9 . . O X . . X O X X X O X . . 9
         8 . . . X O O . X O . . . . . . 8
         7 . O . . X X . . . X . . . . . 7
         6 . . . . . . X . . . X X O . . 6
         5 . . . . O . . . . . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        let expected = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . X . . . . . . . 12
        11 . . . . . . . . . . . . X . X 11
        10 . . . . . O . . . 3 . . O X . 10
         9 . . O X . . X O X X X O X . . 9
         8 . . . X O O . X O 3 . . . . . 8
         7 . O . 3 X X 3 . . X . . . . . 7
         6 . . . . . . X . . . X X O . . 6
         5 . . . . O . . . . . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        assert_eq!(Board::from_str(case).unwrap().to_string(), expected);
    }

    #[test]
    fn recursive_double_three() {
        let case = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . . . . . . . . . . . . . 11
        10 . . . . . . O O . . . . . . . 10
         9 . . . . . . X X . . . . . . . 9
         8 . . . . O X X X X O . . . . . 8
         7 . . . . O X X X X O . . . . . 7
         6 . . . . . . X X . . . . . . . 6
         5 . . . . . . O O . . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        let expected = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . . . . . . . . . . . . . 11
        10 . . . . . . O O . . . . . . . 10
         9 . . . . . 3 X X 3 . . . . . . 9
         8 . . . . O X X X X O . . . . . 8
         7 . . . . O X X X X O . . . . . 7
         6 . . . . . 3 X X 3 . . . . . . 6
         5 . . . . . . O O . . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        assert_eq!(Board::from_str(case).unwrap().to_string(), expected);
    }

}
