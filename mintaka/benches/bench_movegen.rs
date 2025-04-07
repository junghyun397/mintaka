#[feature(test)]

extern crate test;

mod bench_movegen {
    use indoc::indoc;
    use mintaka::game_state::GameState;
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
    fn all_moves(b: &mut Bencher) {
        let case = indoc! {"\
        "};

        bench_movegen!(b, case, pos_unchecked("a1"), pos_unchecked("a1"), pos_unchecked("a1"));
    }

    #[bench]
    fn defend_three_moves(b: &mut Bencher) {
        let case = indoc! {"\
        "};

        bench_movegen!(b, case, pos_unchecked("a1"), pos_unchecked("a1"), pos_unchecked("a1"));
    }

}
