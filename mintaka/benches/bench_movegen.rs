#[feature(test)]

extern crate test;

#[cfg(test)]
mod bench_movegen {
    use indoc::indoc;
    use mintaka::movegen::move_generator::generate_vcf_moves;
    use rusty_renju::board;
    use rusty_renju::board::Board;
    use rusty_renju::notation::pos::pos_unchecked;
    use test::Bencher;

    macro_rules! bench_movegen {
        ($bencher:expr,$case:expr,$player_move:expr,$opponent_move:expr,$new_move:expr) => {{
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

            #$bencher.iter(|| {
                let mut game_state = game_state;

                game_state.set_mut($new_move.into());

                let mut move_picker = MovePicker::new(None, None);

                let acc: Score = 0;
                while let Some((pos, move_eval)) = move_picker.next(&mut game_state) {
                    acc ^= move_eval;
                }

                assert_ne!(acc, 0);
            })
        }}
    }

    #[bench]
    fn vcf_moves(b: &mut Bencher) {
        let board = board!(indoc! {"
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
           A B C D E F G H I J K L M N O"});

        let recent_four = pos_unchecked("i6");

        b.iter(|| {
            let vcf_moves = generate_vcf_moves(&board, 5, recent_four);

            let mut acc = 0;
            for vcf_move in vcf_moves {
                acc ^= vcf_move.idx_usize();
            }

            assert_ne!(acc, 0);
        })
    }

    #[bench]
    fn all_moves(b: &mut Bencher) {
        let board = board!(indoc! {"\
        "});

        bench_movegen!(b, board, pos_unchecked("a1"), pos_unchecked("a1"), pos_unchecked("a1"));
    }

    #[bench]
    fn defend_three_moves(b: &mut Bencher) {
        let board = board!(indoc! {"\
        "});

        bench_movegen!(b, board, pos_unchecked("a1"), pos_unchecked("a1"), pos_unchecked("a1"));
    }

}
