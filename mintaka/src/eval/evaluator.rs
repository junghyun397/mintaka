use crate::game_state::GameState;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::value::Score;

#[cfg(feature = "heuristic-eval")]
pub type ActiveEvaluator = crate::eval::heuristic_evaluator::HeuristicEvaluator;
#[cfg(feature = "nnue-eval")]
pub type ActiveEvaluator = crate::eval::nnue_evaluator::NnueEvaluator;

pub type PolicyDistribution = [Score; pos::BOARD_SIZE];

pub trait Evaluator {

    type EvaluatorParameter;

    fn from_state(state: &GameState) -> Self;

    fn update(&mut self, state: &GameState);

    fn undo(&mut self, state: &GameState, color: Color, pos: Pos);

    fn eval_policy(&self, state: &GameState) -> PolicyDistribution;

    fn eval_value(&self, state: &GameState) -> Score;

}
