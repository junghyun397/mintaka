use mintaka::board::Board;
use crate::eval::score::Score;

pub trait Evaluator : Default {

    fn eval(&self, board: &Board) -> Score;

}
