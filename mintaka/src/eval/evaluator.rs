use rusty_renju::board::Board;
use rusty_renju::notation::value::Eval;

pub trait Evaluator {

    fn eval(&self, board: &Board) -> Eval;

}
