#[cfg(test)]
mod test_eval {
    use indoc::indoc;
    use mintaka::eval::evaluator::{ActiveEvaluator, Evaluator};
    use mintaka::game_state::GameState;
    use rusty_renju::board;
    use rusty_renju::notation::pos;
    use rusty_renju::notation::rule::RuleKind;

    macro_rules! eval {
        ($board:expr) => {{
            let state: GameState<{ RuleKind::Renju }> = $board.into();

            let mut evaluator = ActiveEvaluator::from_state(&state);

            evaluator.eval_value(&state)
        }};
    }

    fn eval_distribution(state: &GameState<{ RuleKind::Renju }>) -> ([f32; pos::BOARD_SIZE], [f32; pos::BOARD_SIZE]) {
        let evaluator = ActiveEvaluator::from_state(state);

        let movegen_field = state.movegen_window.movegen_field & state.board.legal_field(state.board.player_color);

        let mut scores = [f32::NAN; pos::BOARD_SIZE];
        let mut ordering_scores = [f32::NAN; pos::BOARD_SIZE];

        for pos in movegen_field.iter_hot_pos() {
            ordering_scores[pos.idx_usize()] = evaluator.ordering_score(&state.board, pos) as f32;
            let mut state = *state;
            let mut evaluator = evaluator.clone();

            let (artifact, _) = state.play_mut(pos);
            evaluator.play(&state.board, artifact, pos.into());

            let score = -evaluator.eval_value(&state);

            scores[pos.idx_usize()] = score.value_i32() as f32;
        }

        (scores, ordering_scores)
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

        let state: GameState<{ RuleKind::Renju }> = board.into();

        let (scores, ordering_scores) = eval_distribution(&state);

        println!("{:?}, {}", scores, state.board.to_string_with_heatmap(scores, true));
        println!("{:?}, {}", ordering_scores, state.board.to_string_with_heatmap(ordering_scores, true));
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
