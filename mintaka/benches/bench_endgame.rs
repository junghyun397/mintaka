#![feature(test)]

extern crate test;

mod bench_vcf {
    use indoc::indoc;
    use mintaka::config::{Config, SearchObjective};
    use mintaka::eval::evaluator::ActiveEvaluator;
    use mintaka::eval::evaluator::Evaluator;
    use mintaka::game_state::GameState;
    use mintaka::memo::history_table::HistoryTable;
    use mintaka::memo::transposition_table::TranspositionTable;
    use mintaka::search_endgame;
    use mintaka::thread_data::ThreadData;
    use mintaka::thread_type::WorkerThread;
    use mintaka::value::Depth;
    use rusty_renju::board;
    use rusty_renju::history::History;
    use rusty_renju::memo::abstract_transposition_table::AbstractTranspositionTable;
    use rusty_renju::notation::pos::pos_unchecked;
    use rusty_renju::notation::rule::RuleKind;
    use rusty_renju::notation::score::{Score, Scores};
    use rusty_renju::utils::byte_size::ByteSize;
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    use test::Bencher;

    macro_rules! bench_vcf {
        ($bencher:expr,$board:expr,$player_move:expr,$opponent_move:expr,$expect_vcf:expr) => {{
            let config = Config::default();

            let state = GameState {
                board: $board,
                history: {
                    let mut history = History::default();

                    history.set_mut($player_move.into());
                    history.set_mut($opponent_move.into());

                    history
                },
                ..Default::default()
            };

            let evaluator = ActiveEvaluator::from_state(&state);

            let tt = TranspositionTable::new_with_size(ByteSize::from_kib(8));
            let ht = HistoryTable::EMPTY;

            let global_counter_in_1k = AtomicUsize::new(0);
            let aborted = AtomicBool::new(false);

            let td = ThreadData::new(WorkerThread, 0, SearchObjective::Best, config, evaluator, tt.view(), ht, &aborted, &global_counter_in_1k);

            $bencher.iter(|| {
                let result = search_endgame::vcf_search::<{ RuleKind::Renju }>(&mut td.clone(), Depth::MAX, &state, Score::DRAW, -Score::INF, Score::INF);

                tt.clear(1);

                assert_eq!(Score::is_deterministic(result), $expect_vcf)
            })
        }};
    }

    #[bench]
    fn white_vcf(b: &mut Bencher) {
        let case = board!(indoc! {"\
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
        });

        bench_vcf!(b, case, pos_unchecked("h6"), pos_unchecked("i6"), true);
    }

    #[bench]
    fn white_reject_vcf(b: &mut Bencher) {
        let case = board!(indoc! {"\
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . X . . O . . . . . . 13
        12 . . . . . . . O . . . . . . . 12
        11 . . . . O . X . . . . . . . . 11
        10 . O . X . . O X O . . . . . . 10
         9 . . X X O . O X O . . . . . . 9
         8 . . . . X O . X X . . . . . . 8
         7 . . . . X . . . . . . . . . . 7
         6 . . . . . . . O X . . . . . . 6
         5 . . . . . . . . . . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"
        });

        bench_vcf!(b, case, pos_unchecked("h6"), pos_unchecked("i6"), false);
    }

    #[bench]
    fn black_vcf(b: &mut Bencher) {
        let case = board!(indoc! {"\
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
        });

        bench_vcf!(b, case, pos_unchecked("e8"), pos_unchecked("e7"), true);
    }

    #[bench]
    fn black_reject_vcf(b: &mut Bencher) {
        let case = board!(indoc! {"\
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . . . . . . . . . . . . . 11
        10 . . . . . X . . . . . . . . . 10
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
        });

        bench_vcf!(b, case, pos_unchecked("e8"), pos_unchecked("e7"), false);
    }

}
