#![feature(test)]

extern crate test;

mod bench_vcf {
    use indoc::indoc;
    use mintaka::config::Config;
    use mintaka::endgame::vcf_search;
    use mintaka::game_state::GameState;
    use mintaka::memo::history_table::HistoryTable;
    use mintaka::memo::transposition_table::TranspositionTable;
    use mintaka::thread_data::ThreadData;
    use mintaka::thread_type::WorkerThread;
    use rusty_renju::board::Board;
    use rusty_renju::history::History;
    use rusty_renju::memo::abstract_transposition_table::AbstractTranspositionTable;
    use rusty_renju::notation::pos::pos_unchecked;
    use rusty_renju::notation::value::Score;
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    use test::Bencher;

    macro_rules! bench_vcf {
        ($bencher:expr,$case:expr,$score:expr,$player_move:expr,$opponent_move:expr) => {{
            let board = $case.parse::<Board>().unwrap();

            let game_state = GameState {
                board,
                history: {
                    let mut history = History::default();

                    history.set_mut($player_move.into());
                    history.set_mut($opponent_move.into());

                    history
                },
                ..Default::default()
            };

            let config = Config::default();

            let tt = TranspositionTable::new_with_size(512);
            let ht = HistoryTable {};

            let global_counter_in_1k = AtomicUsize::new(0);
            let aborted = AtomicBool::new(false);

            let td = ThreadData::new(WorkerThread, 0, config, tt.view(), ht, &aborted, &global_counter_in_1k);

            $bencher.iter(|| {
                let result = vcf_search::vcf_search(&mut td.clone(), &game_state, u8::MAX);
                assert_eq!(result, $score);
                tt.clear_mut(1);
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

        bench_vcf!(b, case, Some(Score::MAX), pos_unchecked("h6"), pos_unchecked("i6"));
    }

    #[bench]
    fn white_reject_vcf(b: &mut Bencher) {
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

        bench_vcf!(b, case, Some(Score::MAX), pos_unchecked("e8"), pos_unchecked("e7"));
    }

    #[bench]
    fn black_reject_vcf(b: &mut Bencher) {
    }

}

mod bench_vct {

}
