use crate::value::Eval;
use rusty_renju::board::Board;

pub trait Evaluator {

    fn eval(&self, board: &Board) -> Eval;

}
