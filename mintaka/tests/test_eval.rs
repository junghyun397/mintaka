#[cfg(test)]
mod test_eval {
    use indoc::indoc;
    use mintaka::eval::evaluator::{ActiveEvaluator, Evaluator};
    use mintaka::state::GameState;
    use rusty_renju::board;
    use rusty_renju::notation::pos;

    macro_rules! eval {
        ($board:expr) => {{
            let state: GameState = $board.into();

            let mut evaluator = ActiveEvaluator::from_state(&state);

            evaluator.eval_value(&state)
        }};
    }

    fn eval_distribution(state: &GameState) -> [f32; pos::BOARD_SIZE] {
        let mut evaluator = ActiveEvaluator::from_state(state);

        let movegen_field = state.movegen_window.movegen_field & !state.board.legal_field();

        let mut scores = [f32::NAN; pos::BOARD_SIZE];

        for pos in movegen_field.iter_hot_pos() {
            let mut state = *state;
            state.set_mut(pos);

            let score = -evaluator.eval_value(&state);

            scores[pos.idx_usize()] = score as f32;
        }

        scores
    }

    #[test]
    fn eval_map() {
        let board = board!(indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . . . . . . . . . . . . . 11
        10 . . . . . . . . . . . . . . . 10
         9 . . . . . . . . . . . . . . . 9
         8 . . . . . O . X . . . . . . . 8
         7 . . . . . . X . O . . . . . . 7
         6 . . . . . . O X X . . . . . . 6
         5 . . . . . . . O . . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"});

        let state: GameState = board.into();

        let scores = eval_distribution(&state);

        println!("{:?}", scores);
        println!("{}", state.board.to_string_with_heatmap(scores, true));
    }

    #[test]
    fn basic_eval() {
        let board = board!(indoc! {"
           A B C D E F G H I J K L M N O
        15 . . . . . . . . . . . . . . . 15
        14 . . . . . . . . . . . . . . . 14
        13 . . . . . . . . . . . . . . . 13
        12 . . . . . . . . . . . . . . . 12
        11 . . . . . . . . . . . . . . . 11
        10 . . . . . . . . . . . . . . . 10
         9 . . . . . . . . . . . . . . . 9
         8 . . . . . . . X . . . . . . . 8
         7 . . . . . . . . . . . . . . . 7
         6 . . . . . . . . . . . . . . . 6
         5 . . . . . . . . . . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"});

        println!("{}", board.player_color);
        println!("{:?}", eval!(board));
    }

}
