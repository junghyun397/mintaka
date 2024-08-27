use mintaka::board::Board;
use crate::eval::evaluator::Evaluator;
use crate::eval::score::Score;

pub struct HeuristicEvaluator;

impl Default for HeuristicEvaluator {

    fn default() -> Self {
        HeuristicEvaluator
    }

}

impl Evaluator for HeuristicEvaluator {

    fn eval(&self, board: &Board) -> Score {
        todo!()
    }

}

impl HeuristicEvaluator {

}
