use crate::eval::score::Score;
use mintaka::board::Board;

pub trait Evaluator {

    fn eval(&self, board: &Board) -> Score;

}
