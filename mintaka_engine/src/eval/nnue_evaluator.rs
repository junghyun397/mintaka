use crate::eval::evaluator::Evaluator;
use mintaka::board::Board;
use mintaka::notation::node::Score;

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
