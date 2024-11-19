#[cfg(test)]
mod test_endgame_vcf {
    use indoc::indoc;
    use mintaka::board::Board;
    use mintaka::memo::tt_slice_pattern_memo::TTSlicePatternMemo;
    use mintaka::notation::pos::BOARD_SIZE;
    use mintaka_engine::memo::transposition_table::TranspositionTable;
    use mintaka_engine::search::vcf::find_vcf_solution;
    use std::str::FromStr;

    macro_rules! vcf {
        ($case:expr) => {
            let mut board = Board::from_str($case).unwrap();
            let mut tt = TranspositionTable::default();
            let mut memo = TTSlicePatternMemo::default();
            let vcf_result = vcf(&mut tt, &mut memo, &mut board, usize::MAX);
            ""
        };
    }

    #[test]
    fn basic_vcf() {
    }

    #[test]
    fn trap_vcf() {
        todo!()
    }

    #[test]
    fn deep_vcf() {
        let case = indoc! {"
           A B C D E F G H I J K L M N O
        15 O O . . X . X . . O X O O X . 15
        14 O . X . . O . . . . X . . . X 14
        13 O . . . . . . . . . . . . . . 13
        12 O . . . . X . . . . . . . . X 12
        11 X . X . . . . . . . . . . . . 11
        10 O . . O . . X . . . . . . X . 10
         9 O . . . . . . . O . . . . O X 9
         8 . . . O . . O X . . . . X . . 8
         7 . X . . . . . . O . . . X . . 7
         6 . . . . O . . . O . X . . . O 6
         5 . . . X . . . . . . . . . . X 5
         4 X O . . . X . . X . . X . X O 4
         3 . X . . . . . . . . . . . . O 3
         2 . O . . . . . . . . O . . X O 2
         1 X O . O . O . X . X O O X O . 1
           A B C D E F G H I J K L M N O"};

        let expected = indoc! {"
           A B C D E F G H I J K L M N O
        15 O O O . X[X]X . . O X O O X O 15
        14 O O X X . O O O X O X X X O X 14
        13 O O X X X X O X X O X X X O X 13
        12 O O X X O X O X X X O O O O X 12
        11 X X X O X O O O O X X X X O X 11
        10 O O O O X X X X O O X O O X O 10
         9 O X O X X X O O O X O O X O X 9
         8 O X O O X O O X X X O X X X X 8
         7 O X O O O X X O O X O O X O O 7
         6 X O X X O X X O O O X O O X O 6
         5 X X O X X X X O X X X O X X X 5
         4 X O O O X X O X X X O X X X O 4
         3 X X O X X O O O X X X O X O O 3
         2 O O O O X O X X X O O O O X O 2
         1 X O X O O O O X O X O O X O O 1
           A B C D E F G H I J K L M N O"};

        assert_eq!(vcf!(case), expected);

        let case = indoc! {"
           A B C D E F G H I J K L M N O
        15 O . . . X . . . . . . . X . X 15
        14 X . . . . O . . . O . . O . X 14
        13 . . . . . . . O . . . . . O . 13
        12 O . . . . . . . . . . X . . X 12
        11 X . . . . . . . . . . . O . . 11
        10 O . O . . . . . . . . . . . . 10
         9 O O X O . . . . X . . . O . . 9
         8 O . O O . . . X . O . . . . . 8
         7 . X . . . . . . . O . . X . . 7
         6 . . . . . . . . O . . . . . X 6
         5 X . . . . . . . . . . . X . X 5
         4 . . . . . . . . . . . . . X O 4
         3 X . . . . . . . . . . . . X . 3
         2 . . . . . . . X . . . . . . O 2
         1 X O O O . X . . X . X . . . . 1
           A B C D E F G H I J K L M N O"};

        let expected = indoc! {"
           A B C D E F G H I J K L M N O
        15 O . O X X X X O X O X X X O X 15
        14 X X O X X O O X O O X O O O X 14
        13 O[X]O O O O X O X X O O X O O 13
        12 O X X O X O X O O O X X O X X 12
        11 X X X O X O X X X O X X O O X 11
        10 O . O X O X O O O O X X X O X 10
         9 O O X O X O X X X X O X O O O 9
         8 O O O O X O O X O O X O O O X 8
         7 X X O X X O X X O O X X X X O 7
         6 O O X O O O X O O X O X O O X 6
         5 X O X O X X X X O O X O X X X 5
         4 X X X X O O O X X X X O X X O 4
         3 X O X X X O X X O O O O X X O 3
         2 O O O X O X X X O X X O O O O 2
         1 X O O O O X O O X O X X X X O 1
           A B C D E F G H I J K L M N O"};

        assert_eq!(vcf!(case), expected);
    }

}
