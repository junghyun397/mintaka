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
         9 . . . . . . . . . . . . . . . 9
         8 . . . . . . . . . . . . . . . 8
         7 . . . . . . . X . . . . . . . 7
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
