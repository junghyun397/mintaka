#[cfg(test)]
mod test_eval {
    use indoc::indoc;
    use mintaka::eval::evaluator::Evaluator;
    use mintaka::eval::heuristic_evaluator::HeuristicEvaluator;
    use mintaka::game_state::GameState;
    use rusty_renju::board;

    macro_rules! eval {
        ($board:expr) => {{
            let state = GameState::from_board_and_history($board, (&$board).try_into().unwrap());

            HeuristicEvaluator.eval_value(&state.board)
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
        10 . . . . . . . O . . . . . . . 10
         9 . . . . . . . . O . . . . . . 9
         8 . . . . . . X X . O . . . . . 8
         7 . . . . . . X O X . X . . . . 7
         6 . . . . . . . . . O . . . . . 6
         5 . . . . . . . . . . . . . . . 5
         4 . . . . . . . . . . . . . . . 4
         3 . . . . . . . . . . . . . . . 3
         2 . . . . . . . . . . . . . . . 2
         1 . . . . . . . . . . . . . . . 1
           A B C D E F G H I J K L M N O"});

        println!("{:?}", eval!(board));
    }

}
