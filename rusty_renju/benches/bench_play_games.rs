#![feature(test)]

extern crate test;

mod bench_play_games {
    use std::str::FromStr;
    use rusty_renju::board::Board;
    use rusty_renju::history::History;
    use rusty_renju::notation::rule::RuleKind;
    use rusty_renju::utils::empty::Empty;

    macro_rules! bench_game {
        ($bencher:expr,$sequence:literal) => {
            let history = History::from_str($sequence).unwrap();

            $bencher.iter(|| {
                let mut board = Board::<{ RuleKind::Renju }>::empty();

                for pos in history.iter() {
                    let artifact = board.set_mut(pos.unwrap());

                    std::hint::black_box(artifact);
                }

                std::hint::black_box(board);
            })
        };
    }

    #[bench]
    fn bench_game_short_1(b: &mut test::Bencher) {
        bench_game!(b, "h8i9j7g9g8h10j8i8j9j6j11j10i10h9k8l7l9i6k10i12k11k12h11");
    }

    #[bench]
    fn bench_game_short_2(b: &mut test::Bencher) {
        bench_game!(b, "h8h9j6g11g10f10i9e9d8g9g7f6i7e11h7j7i6d12c13i10g8f9d9i5f7");
    }

    #[bench]
    fn bench_game_long_1(b: &mut test::Bencher) {
        bench_game!(b, "h8i9g8h10h6i8i7j8k7j6j9j7h9i10i11j5j4h5k6g7g11h11k5k4i6e8f10h12f7f6e5e6d6c7f5d5f4g3e4g5e7g9i3h2f9e10g4d4f11f8h13e12e11c10g12f13d11c11d10c9c8m7k11i14l11j11l10l9k10m10n11d12c12f14e13m8m11o11g13l8k8g14h14f12i13j12k13j13m12");
    }

    #[bench]
    fn bench_game_long_2(b: &mut test::Bencher) {
        bench_game!(b, "h8g7j8f7j7g8g9h9e6i7h7f8h6j9i10h4g6f6f5e4e7d7f10e8c6e11e10d8c8d6g10h10d10c10d5d11f11d9b11e5f4f12f13j6g12g11h13i14e13g13k8l8l9m10k7k10l11m7m6k13k11h14i15j14k14j10h11e14i11j11l10m11j13i12m9n8l12l13m5l6k5k6");
    }

    #[bench]
    fn bench_game_complex_1(b: &mut test::Bencher) {
        bench_game!(b, "h8h9h7i7j9i8i6j5g7j7k6g5j8j6h4j4j3k5i4h5i5i3l6e5f4g4f5f3g2e3e4d4d3g6c5f7g10i10j11i11i9h10h12f9g9i12e8f8f10e11g11g8e6i13i14e10e12l8l10k9m7m5l4l5n5g13d11b9n9m10l11j10n11n10m6m9k7l7k14m12m11k11k12");
    }

    #[bench]
    fn bench_game_complex_2(b: &mut test::Bencher) {
        bench_game!(b, "h8i9g6g8j9j7i7h7i6i5j4h4g3g5f6h6h5f7e6d6k5j6l6k7l8m7l7l5k2k3f3h3e5e2d4g7c3b2h2i3f4i1f2f5e3");
    }
}
