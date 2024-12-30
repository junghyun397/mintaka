use crate::eval::evaluator::Evaluator;
use rusty_renju::board::Board;
use rusty_renju::notation::node::Score;

pub struct NnueEvaluator {
    weights: (),
}

impl Default for NnueEvaluator {

    fn default() -> Self {
        todo!()
    }

}

impl Evaluator for NnueEvaluator {

    fn eval(&self, board: &Board) -> Score {
        todo!()
    }

}
