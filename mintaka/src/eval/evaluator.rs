use crate::game_state::GameState;
use rusty_renju::board::{Board, MoveArtifact};
use rusty_renju::hash_key::HashKey;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::score::Score;
use rusty_renju::pattern;

#[cfg(not(feature = "neural-eval"))]
pub type ActiveEvaluator<const R: RuleKind> = crate::eval::heuristic_evaluator::HeuristicEvaluator<R>;
#[cfg(feature = "neural-eval")]
pub type ActiveEvaluator<const R: RuleKind> = crate::eval::nnue_evaluator::NnueEvaluator<R>;

pub type PolicyDistribution = [i16; pattern::PATTERN_SIZE];

pub trait Evaluator<const R: RuleKind> {
    type EvaluatorParameter;

    fn require_stabilize(&self) -> bool;

    fn from_state(state: &GameState<R>) -> Self;

    fn init(&mut self, board: &Board<R>);

    fn play(&mut self, board: &Board<R>, artifact: MoveArtifact, plied: MaybePos);

    fn undo(&mut self, board: &Board<R>, artifact: MoveArtifact, removed: MaybePos);

    fn eval_policy(&mut self, state: &GameState<R>) -> PolicyDistribution;

    fn eval_value(&mut self, state: &GameState<R>) -> Score;

    fn hash_key(&self) -> HashKey;
}

pub fn stabilize_eval(current: Score, parent: Score) -> Score {
    (current + parent) / 2
}
