use crate::eval::evaluator::Evaluator;
use crate::game_state::GameState;
use rusty_renju::notation::value::Score;

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

    fn static_eval(&self, state: &GameState) -> Score {
        todo!()
    }

}
