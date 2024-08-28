use crate::formation::FormationLine;
use crate::slice::Slice;

struct PatternInfo {
    pattern: u8,
    formation_pair_line: FormationLine,
    wall: u8,
}

mod pattern {

}

impl Slice {

    pub fn calculate_formation_masks(&self) -> FormationLine {
        todo!()
    }

    fn find_pattern(&self) -> FormationLine {
        todo!()
    }

    fn find_bidirectional_pattern(
        &self,
        black_stones: u8,
        white_stones: u8,
        wall: u8
    ) -> FormationLine {
        todo!()
    }

}
