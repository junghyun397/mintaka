#![feature(test)]

extern crate test;

mod bench_vcf {
    use indoc::indoc;
    use mintaka::config::Config;
    use mintaka::endgame::vcf;
    use mintaka::memo::history_table::HistoryTable;
    use mintaka::memo::transposition_table::TranspositionTable;
    use mintaka::thread_data::ThreadData;
    use mintaka::thread_type::ThreadType;
    use rusty_renju::board::Board;
    use rusty_renju::memo::abstract_transposition_table::AbstractTranspositionTable;
    use rusty_renju::notation::value::Score;
    use rusty_renju::utils::platform;
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    use test::Bencher;

    macro_rules! bench_vcf {
        ($bencher:expr,$case:expr,$score:expr) => {{
            let board = $case.parse::<Board>().unwrap();

            let config = Config::default();

            let tt = TranspositionTable::new_with_size(512);
            let ht = HistoryTable {};

            let global_counter_in_1k = AtomicUsize::new(0);
            let global_aborted = AtomicBool::new(false);

            let td = ThreadData::new(ThreadType::Main, 0, config, tt.view(), ht, &global_aborted, &global_counter_in_1k);

            $bencher.iter(|| {
                let result = vcf::vcf_search(&mut td.clone(), &board, u8::MAX);
                assert_eq!(result, $score);
                tt.clear_mut(platform::available_cores());
            })
        }};
    }

    #[bench]
    fn white_vcf(b: &mut Bencher) {
        let case = indoc! {"\
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
           A B C D E F G H I J K L M N O"
        };

        bench_vcf!(b, case, Score::MAX);
    }

    #[bench]
    fn black_vcf(b: &mut Bencher) {
        let case = indoc! {"\
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
           A B C D E F G H I J K L M N O"
        };

        bench_vcf!(b, case, Score::MAX);
    }
}

mod bench_vct {

}
