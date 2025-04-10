use crate::notation::color::{AlignedColorContainer, Color};
use crate::notation::value::Score;
use crate::pattern::Pattern;

impl Pattern {

    #[inline(always)]
    pub fn calculate_pattern_score<const C: Color>(&self) -> Score {
        let closed_fours = self.count_closed_fours(); // 2 instructions
        let mut masked = self.apply_mask(0b1); // 1 instruction

        // TODO

        PATTERN_SCORE_LUT.player_unit::<C>()[closed_fours as usize][masked as usize] as Score
    }

}

type PatternScoreLUT = [[i8; u8::MAX as usize]; 3];

const PATTERN_SCORE_LUT: AlignedColorContainer<PatternScoreLUT> = build_pattern_score_lut();

const fn build_pattern_score_lut() -> AlignedColorContainer<PatternScoreLUT> {
    let mut lut = AlignedColorContainer::new(
        [[0; u8::MAX as usize]; 3],
        [[0; u8::MAX as usize]; 3],
    );

    // TODO

    lut
}
