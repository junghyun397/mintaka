use crate::eval::evaluator::Evaluator;
use crate::eval::score::Score;
use mintaka::board::Board;

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
