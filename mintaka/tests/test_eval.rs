#[cfg(test)]
mod test_eval {
    use indoc::indoc;
    use mintaka::eval::evaluator::{ActiveEvaluator, Evaluator};
    use mintaka::game_state::GameState;
    use rusty_renju::board;

    macro_rules! eval {
        ($board:expr) => {{
            let state: GameState = $board.into();

            let mut evaluator = ActiveEvaluator::from_state(&state.clone());

            evaluator.eval_value(&state)
        }};
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
         9 . . . . . . X . O . . . . . . 9
         8 . . . . . . . X . . . . . . . 8
         7 . . . . . . X O O O . . . . . 7
         6 . . . . . . . X X X O . . . . 6
         5 . . . . . . . . O . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"});

        println!("{}", board.to_string_with_pattern_analysis());

        println!("{:?}", eval!(board));
    }

}
