use crate::eval::evaluator::{Evaluator, PolicyDistribution};
use crate::game_state::GameState;
use rusty_renju::notation::value::Score;

pub struct HeuristicEvaluator;

impl Default for HeuristicEvaluator {

    fn default() -> Self {
        HeuristicEvaluator
    }

}

impl Evaluator for HeuristicEvaluator {

    fn eval_value(&self, _state: &GameState) -> Score {
        0
    }

    fn eval_policy(&self, _state: &GameState) -> PolicyDistribution {
        todo!()
    }

}
