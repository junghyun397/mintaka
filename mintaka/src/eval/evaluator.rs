use rusty_renju::board::Board;
use rusty_renju::notation::pos;
use rusty_renju::notation::value::Score;

pub type PolicyDistribution = [f32; pos::BOARD_SIZE];

pub trait Evaluator {

    const POLICY_EVALUATION: bool;

    fn eval_value(&self, board: &Board) -> Score;

    fn eval_policy(&self, board: &Board) -> PolicyDistribution;

}
