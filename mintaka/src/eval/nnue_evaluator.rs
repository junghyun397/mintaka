use crate::eval::evaluator::{Evaluator, PolicyDistribution};
use crate::eval::nn::network_params::NnueNetworkParams;
use crate::game_state::GameState;
use rusty_renju::board::Board;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
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
pub struct NnueEvaluator {
    black_network: (),
    white_network: (),
    hash_key: HashKey,
}

impl Evaluator for NnueEvaluator {
    type EvaluatorParameter = NnueNetworkParams;

    fn from_state(state: &GameState) -> Self {
        todo!()
    }

    fn init(&mut self, board: &Board) {
        todo!()
    }

    fn play(&mut self, board: &Board, pos: Pos) {
        todo!()
    }

    fn undo(&mut self, board: &Board, pos: Pos) {
        todo!()
    }

    fn eval_policy(&mut self, state: &GameState) -> PolicyDistribution {
        todo!()
    }

    fn eval_value(&mut self, state: &GameState) -> Score {
        todo!()
    }

    fn hash_key(&self) -> HashKey {
        self.hash_key
    }
}

impl NnueEvaluator {

    fn shape_inputs(&self, board: &Board) -> [i8; pos::BOARD_SIZE] {
        todo!()
    }

}
