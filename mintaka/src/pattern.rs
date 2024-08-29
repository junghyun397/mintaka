use crate::notation::rule;
use crate::slice::Slice;

pub struct FormationPatch {
    pub black_patch: u8,
    pub white_patch: u8
}

pub type SlicePatch = [FormationPatch; rule::U_BOARD_WIDTH];

struct PatternInfo {
    pattern: u8,
    formation_pair_line: SlicePatch,
    wall: u8,
}

impl Slice {

    pub fn calculate_formation_patch(&self) -> SlicePatch {
        todo!()
    }

    fn find_pattern(&self) -> SlicePatch {
        todo!()
    }

    fn find_bidirectional_pattern(
        &self,
        black_stones: u8,
        white_stones: u8,
        wall: u8
    ) -> SlicePatch {
        todo!()
    }

}

fn increase_closed_four(encoded: u8) -> u8 {
    encoded | (0b1000_000 >> (encoded >> 7))
}
