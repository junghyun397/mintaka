use crate::board::Board;
use crate::movegen::move_generator::MoveGenerator;
use crate::notation::pos::Pos;

struct NarrowMoveGenerator;

impl MoveGenerator for NarrowMoveGenerator {

    fn generate_moves(board: &Board) -> Vec<Pos> {
        todo!()
    }

}
