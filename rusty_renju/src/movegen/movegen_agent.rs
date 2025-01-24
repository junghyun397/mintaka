use crate::bitfield::Bitfield;
use crate::notation::pos::Pos;

pub struct MovegenAgent {
    neighborhood_field: Bitfield,
    begin: Pos,
    end: Pos
}

impl MovegenAgent {

    pub fn set_mut(&mut self, pos: Pos) {
    }

    pub fn unset_mut(&mut self, pos: Pos) {
    }

}
