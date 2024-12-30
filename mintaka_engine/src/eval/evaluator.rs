use rusty_renju::board::Board;
use rusty_renju::notation::node::Score;

pub trait Evaluator {

    fn eval(&self, board: &Board) -> Score;

}
