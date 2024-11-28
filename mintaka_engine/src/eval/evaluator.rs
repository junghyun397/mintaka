use mintaka::board::Board;
use mintaka::notation::node::Score;

pub trait Evaluator {

    fn eval(&self, board: &Board) -> Score;

}
