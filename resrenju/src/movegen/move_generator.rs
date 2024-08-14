use crate::board::Board;
use crate::notation::pos::Pos;

pub trait MoveGenerator {

    fn generate_moves(board: &Board) -> Vec<Pos> {
        todo!()
    }

}
