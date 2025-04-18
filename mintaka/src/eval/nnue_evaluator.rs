use crate::eval::evaluator::{Evaluator, PolicyDistribution};
use crate::game_state::GameState;
use rusty_renju::notation::value::Score;

pub struct NnueEvaluator {
    black_network: (),
    white_network: ()
}

impl Default for NnueEvaluator {

    fn default() -> Self {
        todo!()
    }

}

impl Evaluator for NnueEvaluator {

    const POLICY_EVALUATION: bool = true;

    fn eval_value(&self, state: &GameState) -> Score {
        todo!()
    }

    fn eval_policy(&self, state: &GameState) -> PolicyDistribution {
        todo!()
    }

}
