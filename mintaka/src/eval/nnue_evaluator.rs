use crate::eval::evaluator::{Evaluator, PolicyDistribution};
use crate::game_state::GameState;
use rusty_renju::board::Board;
use rusty_renju::notation::pos;
use rusty_renju::notation::value::Score;

struct NNUEInput {
    threes: [i8; 256],
    closed_fours: [i8; 256],
    open_fours: [i8; 256],
    fork_or_forbidden_moves: [i8; 256],
    player_stones: [i8; 256],
    opponent_stones: [i8; 256]
}

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

impl NnueEvaluator {

    fn shape_inputs(&self, board: Board) -> [i8; pos::BOARD_SIZE] {
        todo!()
    }

}
