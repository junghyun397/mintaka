use crate::memo::tt_entry::Score;
use mintaka::board::Board;

pub trait Evaluator {

    fn eval(&self, board: &Board) -> Score;

}
