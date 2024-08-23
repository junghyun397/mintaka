use crate::formation::FormationPairLine;
use crate::slice::Slice;

struct PatternInfo {
    pattern: u8,
    formation_pair_line: FormationPairLine,
    wall: u8,
}

mod pattern {

}

impl Slice {

    pub fn calculate_formation_masks(&self) -> FormationPairLine {
        todo!()
    }

    fn find_pattern(&self) -> FormationPairLine {
        todo!()
    }

    fn find_bidirectional_pattern(
        &self,
        black_stones: u8,
        white_stones: u8,
        wall: u8
    ) -> FormationPairLine {
        todo!()
    }

}
