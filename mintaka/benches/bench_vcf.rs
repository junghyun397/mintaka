#![feature(test)]

extern crate test;

mod bench_vcf {
    use indoc::indoc;
    use mintaka::endgame::vcf;
    use mintaka::memo::transposition_table::TranspositionTable;
    use rusty_renju::board::Board;
    use rusty_renju::memo::tt_slice_pattern_memo::TTSlicePatternMemo;
    use std::str::FromStr;
    use test::Bencher;

    #[bench]
    fn bench_simple_vcf(bencher: &mut Bencher) {
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

        let mut board = Board::from_str(case).unwrap();

        bencher.iter(|| { std::hint::black_box(for i in 0 .. 100 {
            let mut tt = TranspositionTable::new_with_size(1);
            let mut memo = TTSlicePatternMemo::default();
            let result = vcf::vcf_sequence(&mut tt, &mut memo, &mut board, u8::MAX).unwrap();
        }); });
    }

}

