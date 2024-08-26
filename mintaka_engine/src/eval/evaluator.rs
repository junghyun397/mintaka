use mintaka::board::Board;
use crate::eval::score::Score;

trait Evaluator {

    fn eval(board: &Board) -> Score;

}
