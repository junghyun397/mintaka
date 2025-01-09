use crate::eval::evaluator::Evaluator;
use crate::value::Eval;
use rusty_renju::board::Board;
use rusty_renju::board_iter::BoardIterItem;

pub struct HeuristicEvaluator;

impl Default for HeuristicEvaluator {

    fn default() -> Self {
        HeuristicEvaluator
    }

}

impl Evaluator for HeuristicEvaluator {

    fn eval(&self, board: &Board) -> Eval {
        let mut score: Eval = 0;

        for item in board.iter_items() {
            score += match item {
                BoardIterItem::Pattern(pattern) => {
                    let (player_pattern, opponent_pattern) = pattern.access_unit_pair(board.player_color);

                    0
                }
                _ => 0
            }

        }

        score
    }

}
