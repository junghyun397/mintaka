use crate::eval::evaluator::{Evaluator, PolicyDistribution};
use crate::eval::nn::network_params::NnueNetworkParams;
use crate::game_state::GameState;
use rusty_renju::board::{Board, MoveArtifact};
use rusty_renju::hash_key::HashKey;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::score::Score;

struct NNUEInput {
    threes: [i8; 256],
    closed_fours: [i8; 256],
    open_fours: [i8; 256],
    fork_or_forbidden_moves: [i8; 256],
    player_stones: [i8; 256],
    opponent_stones: [i8; 256]
}

#[derive(Clone)]
pub struct NnueEvaluator<const R: RuleKind> {
    black_network: (),
    white_network: (),
    hash_key: HashKey,
}

impl<const R: RuleKind> Evaluator<R> for NnueEvaluator<R> {
    type EvaluatorParameter = NnueNetworkParams;

    fn require_stabilize(&self) -> bool {
        false
    }

    fn from_state(state: &GameState<R>) -> Self {
        todo!()
    }

    fn init(&mut self, board: &Board<R>) {
        todo!()
    }

    fn play(&mut self, board: &Board<R>, artifact: MoveArtifact, pos: MaybePos) {
        todo!()
    }

    fn undo(&mut self, board: &Board<R>, artifact: MoveArtifact, pos: MaybePos) {
        todo!()
    }

    fn eval_policy(&mut self, state: &GameState<R>) -> PolicyDistribution {
        todo!()
    }

    fn eval_value(&mut self, state: &GameState<R>) -> Score {
        todo!()
    }

    fn hash_key(&self) -> HashKey {
        self.hash_key
    }
}

impl<const R: RuleKind> NnueEvaluator<R> {
    fn shape_inputs(&self, board: &Board<R>) -> [i8; pos::BOARD_SIZE] {
        todo!()
    }
}
