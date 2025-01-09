use crate::eval::evaluator::Evaluator;
use crate::value::Eval;
use rusty_renju::board::Board;

pub struct NnueEvaluator {
    weights: (),
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
