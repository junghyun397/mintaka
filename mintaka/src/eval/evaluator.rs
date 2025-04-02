use rusty_renju::board::Board;
use rusty_renju::notation::value::Score;

pub trait Evaluator {

    fn static_eval(&self, board: &Board) -> Score;

}
