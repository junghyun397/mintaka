use crate::const_for;
use crate::notation::color::{AlignedColorContainer, Color, ColorContainer};
use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::pattern::Pattern;

#[derive(Debug, Copy, Clone)]
pub struct ScoreTable {
    pub positions: AlignedColorContainer<[i8; pos::BOARD_SIZE]>,
}

impl Default for ScoreTable {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl ScoreTable {

    pub const EMPTY: Self = unsafe { std::mem::zeroed() };

}

impl ScoreTable {

    fn update_position_mut<const C: Color>(&mut self, idx: usize, pattern: Pattern) {
        self.positions.get_ref_mut::<C>()[idx] =
            PATTERN_SCORE_LUT.get_ref::<C>()[encode_pattern_to_score_key(pattern)]
    }

    fn clear_position_mut<const C: Color>(&mut self, idx: usize) {
        self.positions.get_ref_mut::<C>()[idx] = 0;
    }

    fn add_neighborhood_score_mut(&mut self, pos: Pos) {
        todo!()
    }

    fn remove_neighborhood_score_mut(&mut self, pos: Pos) {
        todo!()
    }

}

fn encode_pattern_to_score_key(pattern: Pattern) -> usize {
    let mut pattern_key = pattern.count_closed_fours() & 0b11; // 3 instructions

    pattern_key |= (pattern.count_open_fours() & 0b11) << 2; // 4 instructions
    pattern_key |= (pattern.count_open_threes() & 0b11) << 4; // 4 instructions
    pattern_key |= (pattern.has_five() as u32) << 6; // 4 instructions
    pattern_key |= (pattern.has_overline() as u32) << 7; // 4 instructions

    pattern_key as usize // 19 instructions
}

struct HeuristicPositionScores; impl HeuristicPositionScores {
    const OPEN_THREE: i8 = 5;
    const CLOSED_FOUR: i8 = 2;
    const OPEN_FOUR: i8 = 80;
    const DOUBLE_THREE_FORK: i8 = 30;
    const THREE_FOUR_FORK: i8 = 50;
    const DOUBLE_FOUR_FORK: i8 = 80;
    const FIVE: i8 = 125;
    const DOUBLE_THREE_FORBID: i8 = 1;
    const DOUBLE_FOUR_FORBID: i8 = -2;
    const OVERLINE_FORBID: i8 = -2;
}

const PATTERN_SCORE_LUT: ColorContainer<[i8; 0b1 << 7]> = build_pattern_score_lut();

const fn build_pattern_score_lut() -> ColorContainer<[i8; 128]> {
    let mut acc = ColorContainer::new(
        [0; 0b1 << 7],
        [0; 0b1 << 7]
    );

    const fn flash_score_variants(
        color: Color,
        lut: &mut [i8; 0b1 << 7],
    ) {
        const_for!(pattern_key in 0, 0b1 << 7; {
            let closed_fours = pattern_key & 0b11;
            let open_fours = (pattern_key & 0b1100) >> 2;
            let open_threes = (pattern_key & 0b110000) >> 4;
            let has_five = (pattern_key & 0b1000000) >> 6;
            let has_overline = (pattern_key & 0b10000000) >> 7;

            lut[pattern_key] = match color {
                Color::Black => {
                    if has_five != 0 {
                        HeuristicPositionScores::FIVE
                    } else if has_overline > 0 {
                        HeuristicPositionScores::OVERLINE_FORBID
                    } else if closed_fours + open_fours > 1 {
                        HeuristicPositionScores::DOUBLE_FOUR_FORBID
                    } else if open_threes > 1 {
                        HeuristicPositionScores::DOUBLE_THREE_FORBID
                    } else if open_fours == 1 {
                        HeuristicPositionScores::OPEN_FOUR
                    } else if closed_fours == 1 && open_threes == 1 {
                        HeuristicPositionScores::THREE_FOUR_FORK
                    } else if open_threes == 1 {
                        HeuristicPositionScores::OPEN_THREE
                    } else {
                        0
                    }
                },
                Color::White => {
                    if has_five != 0 {
                        HeuristicPositionScores::FIVE
                    } else if open_fours > 0 {
                        HeuristicPositionScores::OPEN_FOUR
                    } else if closed_fours > 1 {
                        HeuristicPositionScores::DOUBLE_FOUR_FORK
                    } else if closed_fours > 0 && open_threes > 0 {
                        HeuristicPositionScores::THREE_FOUR_FORK
                    } else if open_threes > 1 {
                        HeuristicPositionScores::DOUBLE_THREE_FORK
                    } else if open_threes == 1 {
                        HeuristicPositionScores::OPEN_THREE
                    } else if closed_fours == 1 {
                        HeuristicPositionScores::CLOSED_FOUR
                    } else {
                        0
                    }
                }
            }
        });
    }

    flash_score_variants(Color::Black, &mut acc.black);
    flash_score_variants(Color::White, &mut acc.white);

    acc
}
