use crate::eval::evaluator::Evaluator;
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

    fn eval(&self, board: &Board) -> i16 {
        todo!()
    }

}
