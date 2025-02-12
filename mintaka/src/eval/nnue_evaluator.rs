use crate::eval::evaluator::Evaluator;
use rusty_renju::board::Board;
use rusty_renju::notation::value::Eval;

pub struct NnueEvaluator {
    black_weights: (),
    white_weights: (),
    general_weights: (),
}

impl Default for NnueEvaluator {

    fn default() -> Self {
        todo!()
    }

}

impl Evaluator for NnueEvaluator {

    fn eval(&self, board: &Board) -> Eval {
        todo!()
    }

}
