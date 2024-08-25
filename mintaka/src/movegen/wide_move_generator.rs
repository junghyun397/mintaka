use crate::board::Board;
use crate::movegen::move_generator::MoveGenerator;
use crate::notation::pos::Pos;

struct WideMoveGenerator;

impl MoveGenerator for WideMoveGenerator {

    fn generate_moves(board: &Board) -> Vec<Pos> {
        todo!()
    }

}
