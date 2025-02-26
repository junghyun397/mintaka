#[cfg(test)]
mod test_vcf {
    use indoc::indoc;
    use mintaka::config::Config;
    use mintaka::endgame::vcf;
    use mintaka::memo::history_table::HistoryTable;
    use mintaka::memo::transposition_table::TranspositionTable;
    use mintaka::thread_data::ThreadData;
    use mintaka::thread_type::ThreadType;
    use rusty_renju::board::Board;
    use rusty_renju::memo::abstract_transposition_table::AbstractTranspositionTable;
    use std::sync::atomic::{AtomicBool, AtomicUsize};

    macro_rules! vcf {
        ($case:expr) => {{
            let mut board = $case.parse::<Board>().unwrap();

            let config = Config::default();

            let tt = TranspositionTable::new_with_size(512);
            let ht = HistoryTable {};

            let global_counter_in_1k = AtomicUsize::new(0);
            let global_aborted = AtomicBool::new(false);

            let mut td = ThreadData::new(ThreadType::Main, 0, config, &tt, ht, &global_aborted, &global_counter_in_1k);
            let vcf_result = vcf::vcf_sequence(&mut td, &board, u8::MAX).unwrap();
            let time = td.running_time();

            board.batch_set_mut(&vcf_result.clone().into_boxed_slice());
            let board_string = board.to_string_with_move_marker(vcf_result.last().copied().unwrap());

            println!("{}", board_string);
            println!("length: {}", vcf_result.len());
            println!("sequence: {:?}", vcf_result);
            println!("time: {:?}", time);
            println!("hash usage: {}", tt.hash_usage());
            println!("nodes: {}", td.batch_counter.count_local_total());

            board_string
        }};
    }

    #[test]
    fn basic_vcf() {
        let case = indoc! {"}
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . . . . . . . . O . . . . 11
        10 . . . . . . . . . X . X . . . 10
         9 . . . . . . . . . O . . . . . 9
         8 . . . . . . . X . X X O . . . 8
         7 . . . . . . X . X O . . . . . 7
         6 . . . . . . . O O . . . . . . 6
         5 . . . . . . O . . . . . . . . 5
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
        11 . . . . . . . . . . O . O . . 11
        10 . . . . . . . .[X]X . X . . . 10
         9 . . . . . . . . X O X . . . . 9
         8 . . . . . . O X X X X O . . . 8
         7 . . . . . . X . X O 3 . . . . 7
         6 . . . . . O . O O . . . . . . 6
         5 . . . . . . O . . . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        assert_eq!(vcf!(case), expected);

        let case = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . . . . . . . O . . . . . 11
        10 . . . . . X . . . X . . . . . 10
         9 . . . . . O . O . . . O . . . 9
         8 . . O X X . . X . . X . . . . 8
         7 . . . . O . . . . X . . . . . 7
         6 . . . . . X O . . . . O . . . 6
         5 . . . . . . . . . . . . . . . 5
         4 . . . . . . . X . . . . . . . 4
         3 . . . . . . O . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        let expected = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . . . . . . . O . . . . . 11
        10 . . . . . X . . . X . . . . . 10
         9 . . . . . O O O X O . O . . . 9
         8 . . O X X O X X O X X . . . . 8
         7 . . . . O . O . X X[X]. . . . 7
         6 . . . . . X O O O X . O . . . 6
         5 . . . . . . X . X O O . . . . 5
         4 . . . . . X O X X X O X . . . 4
         3 . . . . . . O . X . . . . . . 3
         2 . . . . . . . . O O . . . . . 2
         1 . . . . . . . . X . . . . . . 1
           A B C D E F G H I J K L M N O"};

        assert_eq!(vcf!(case), expected);

        let case = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . . . . . O . . . . . . . 11
        10 . . . . . . . . . X O . . . . 10
         9 . . . . . . . O X . X . . . . 9
         8 . . . . . . . X O . X . O . . 8
         7 . . . . . . O . . O . X . . . 7
         6 . . . . . . . . . X . O O X . 6
         5 . . . . . . . . X O . X X . . 5
         4 . . . . . . . O . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        let expected = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . O . . . O . . . 12
        11 . . . . . . . O X O X X X X O 11
        10 . . . . . . . . . X O O[X]. . 10
         9 . . . . . . . O X O X X X X O 9
         8 . . . . . . . X O . X X O . . 8
         7 . . . . . . O . . O O X O . . 7
         6 . . . . . . . . . X . O O X . 6
         5 . . . . . . . . X O . X X . . 5
         4 . . . . . . . O . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        assert_eq!(vcf!(case), expected);

        let case = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . X . . . . . . . . 12
        11 . . . . . . . . . . X . . . . 11
        10 . . . . . . . O . . . . . . . 10
         9 . . . . O . . X . . O . . . . 9
         8 . . . . X O X X X . . . . . . 8
         7 . . . O . O O . . X . . . . . 7
         6 . . . . . . . . . . O . . . . 6
         5 . . . . . . . . . O X . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        let expected = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . O . . . . . . . . . 13
        12 . . . . . . X . O . O . . . . 12
        11 . . . . . O O X . X X[X]. . . 11
        10 . . . . . . X O X O . . . . . 10
         9 . . . . O X X X O X O . . . . 9
         8 . . . . X O X X X X O . . . . 8
         7 . . . O . O O . O X . . . . . 7
         6 . . . . . . . . . X O . . . . 6
         5 . . . . . . . . . O X . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        assert_eq!(vcf!(case), expected);

        let case = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . . . . . . . . . . . . . 11
        10 . . . . . . . . . . . . . . . 10
         9 . . . . O . . . O . . . . . . 9
         8 . . . . . . . X O O . . . . . 8
         7 . . . . O X . X . . . . . . . 7
         6 . . . . . X . . O X . . . . . 6
         5 . . . . . . . . . . . . . . . 5
         4 . . . . . . . X . . . X . . . 4
         3 . . . . . . O . . . . . . . . 3
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
        10 . . . . . X . O . . . . . . . 10
         9 . . . . O O O X O . . . . . . 9
         8 . . . . . X[X]X O O . . . . . 8
         7 . . . . O X X X X O . . . . . 7
         6 . . . . . X . X O X . . . . . 6
         5 . . . . X O X O O . O . . . . 5
         4 . . . O . X X X O X . X . . . 4
         3 . . . . O . O . X . . . . . . 3
         2 . . . . . . . . . O . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        assert_eq!(vcf!(case), expected);
    }

    #[test]
    fn trap_vcf() {
        let case = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . O X . . O . . . . . . 13
        12 . . . . . . . O . . . . . . . 12
        11 . . . . O . X . . . . . . . . 11
        10 . O . X . . O X O . . . . . . 10
         9 . . X X O . O X O . . . . . . 9
         8 . . . X X O . X X . . . . . . 8
         7 . . . . X . . . . . . . . . . 7
         6 . . . . . . . O X . . . . . . 6
         5 . . . . . . . . . . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        let expected = indoc! {"
           A B C D E F G H I J K L M N O
        15 X . .[O]O . . . . . . . . . . 15
        14 . O X O X O . . . . . . . . . 14
        13 X O O O O X X . O . . . . . . 13
        12 4 O X O O O X O X . . . . . . 12
        11 X X X 4 O O X . O . . . . . . 11
        10 . O 4 X X X O X O . . . . . . 10
         9 . . X X O 4 O X O . . . . . . 9
         8 . . . X X O X X X . . . . . . 8
         7 . . . . X . O 4 . . . . . . . 7
         6 . . . . . . O O X . . . . . . 6
         5 . . . . . . . . X . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        assert_eq!(vcf!(case), expected);

        let case = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . X . . . . . . . 15
        14 . . . . . X O . . O . . . . . 14
        13 . . . . X . O . . O . X . . . 13
        12 . . . . . . O . X O . . . . . 12
        11 . . . . O X X X O X O . . . . 11
        10 . . . X O O O O X O X X . . . 10
         9 . . . . . . X O X . . O X . . 9
         8 . . . . . . . X . . . . . . . 8
         7 . . . . . . . . . . . . . . . 7
         6 . . . . . . . . . . . . . . . 6
         5 . . . . . . . . . . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        let expected = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . X X O . X . . . . 15
        14 . . . . . X O O X O O X . O . 14
        13 . . . . X . O X O O O X X . . 13
        12 . . . . . X O O X O O O[O]4 . 12
        11 . . . . O X X X O X O . X . . 11
        10 . . . X O O O O X O X X . . . 10
         9 . . . . . . X O X . X O X . . 9
         8 . . . . . . . X . . . . . . . 8
         7 . . . . . . . . . . . . . . . 7
         6 . . . . . . . . . . . . . . . 6
         5 . . . . . . . . . . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        assert_eq!(vcf!(case), expected);

        let case = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . O . . . O . . . 14
        13 . . . . . . . X . . X . . . . 13
        12 . . . . . . . . O . . . . . . 12
        11 . . . O . X O X . . . . . . . 11
        10 . . . X . . . . X . . . . . . 10
         9 . . . . . . . . . . . . . . . 9
         8 . . . . X . O X O . X . . . . 8
         7 . . . . . . O O X . . . . . . 7
         6 . . . . . . . . . . O . . . . 6
         5 . . . . . X . . . . X . O . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        let expected = indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . O . . . O . . . 14
        13 . . . . . . . X . . X . . . . 13
        12 . . . . . O[O]6 O O . . . . . 12
        11 . . . O . X O X . X . . . . . 11
        10 . . . X X 6 X X X O X . . . . 10
         9 . . . . . O O X O O . . . . . 9
         8 . . . . X . O X O O X . . . . 8
         7 . . . . . . O O X X X . . . . 7
         6 . . . . . . O X O O O O X . . 6
         5 . . . . . X X . . X X . O . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"};

        assert_eq!(vcf!(case), expected);
    }

    #[test]
    fn deep_vcf() {
        // 5 ms
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

        // 9 ms
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
