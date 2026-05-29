use crate::game_state::GameState;
use rusty_renju::board::{Board, MoveArtifact};
use rusty_renju::memo::hash_key::HashKey;
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

    fn init(&mut self, board: &Board);

    fn play(&mut self, board: &Board, artifact: MoveArtifact, plied: Pos);

    fn undo(&mut self, board: &Board, artifact: MoveArtifact, removed: Pos);

    fn eval_policy(&mut self, state: &GameState) -> PolicyDistribution;

    fn eval_value(&mut self, state: &GameState) -> Score;

    fn hash_key(&self) -> HashKey;
}

pub fn stabilize_eval(current: Score, parent: Score) -> Score {
    (current + parent) / 2
}
