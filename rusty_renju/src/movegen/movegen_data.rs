use crate::notation::pos;
use crate::notation::pos::Pos;

pub struct MovegenData {
    score_field: [u8; pos::BOARD_SIZE],
    total_score: i16,
    begin: Pos,
    end: Pos
}

impl MovegenData {

    pub fn set_mut(&mut self, pos: Pos) {
    }

    pub fn unset_mut(&mut self, pos: Pos) {
    }

}
