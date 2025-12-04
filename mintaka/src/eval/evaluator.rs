use crate::game_state::GameState;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::score::Score;
use rusty_renju::pattern;

#[cfg(not(feature = "neural-eval"))]
pub type ActiveEvaluator = crate::eval::heuristic_evaluator::HeuristicEvaluator;
#[cfg(feature = "neural-eval")]
pub type ActiveEvaluator = crate::eval::nnue_evaluator::NnueEvaluator;

pub type PolicyDistribution = [i16; pattern::PATTERN_SIZE];

pub trait Evaluator {

    type EvaluatorParameter;

    fn from_state(state: &GameState) -> Self;

    fn update(&mut self, state: &GameState);

    fn undo(&mut self, state: &GameState, color: Color, pos: Pos);

    fn eval_policy(&mut self, state: &GameState) -> PolicyDistribution;

    fn eval_value(&mut self, state: &GameState) -> Score;

}
