use crate::board::Board;
use crate::movegen::move_generator::MoveGenerator;
use crate::notation::pos::Pos;

struct ThreatMoveGenerator;

impl MoveGenerator for ThreatMoveGenerator {

    fn generate_moves(board: &Board) -> Vec<Pos> {
        todo!()
    }

}
