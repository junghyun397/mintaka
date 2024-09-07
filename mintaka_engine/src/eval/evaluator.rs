use mintaka::board::Board;

pub trait Evaluator {

    fn eval(&self, board: &Board) -> i16;

}
