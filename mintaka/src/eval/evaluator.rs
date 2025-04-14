use crate::game_state::GameState;
use rusty_renju::notation::pos;
use rusty_renju::notation::value::Score;

pub type PolicyDistribution = [f32; pos::BOARD_SIZE];

pub trait Evaluator {

    fn eval_value(&self, state: &GameState) -> Score;

    fn eval_policy(&self, state: &GameState) -> PolicyDistribution;

}
