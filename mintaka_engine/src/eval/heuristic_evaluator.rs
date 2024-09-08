use mintaka::board::Board;
use crate::eval::evaluator::Evaluator;

pub struct HeuristicEvaluator;

impl Default for HeuristicEvaluator {

    fn default() -> Self {
        HeuristicEvaluator
    }

}

impl Evaluator for HeuristicEvaluator {

    fn eval(&self, board: &Board) -> i16 {
        todo!()
    }

}

impl HeuristicEvaluator {

}
