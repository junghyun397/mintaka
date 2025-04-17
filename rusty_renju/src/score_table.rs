use crate::notation::color::{Color, ColorContainer};
use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::pattern::Pattern;
use crate::{const_for, slice};
use std::simd::Simd;

#[derive(Copy, Clone)]
pub struct SlicePatternCount {
    pub threes: u8,
    pub fours: u8,
}

impl SlicePatternCount {

    pub const EMPTY: Self = Self {
        threes: 0,
        fours: 0,
    };

}

#[derive(Copy, Clone)]
pub struct ScoreTable {
    pub slice_pattern_counts: ColorContainer<[SlicePatternCount; slice::TOTAL_SLICE_AMOUNT]>,
    pub position_scores: ColorContainer<[i8; pos::BOARD_SIZE]>,
}

pub trait ScoreTableOps {

    fn set_slice_mut<const C: Color>(&mut self, idx: usize, threes: u8, fours: u8);

    fn clear_slice_mut<const C: Color>(&mut self, idx: usize);

    fn sum_slices<const C: Color>(&self) -> SlicePatternCount;

    fn update_position_mut<const C: Color>(&mut self, idx: usize, pattern: Pattern);

    fn clear_position_mut<const C: Color>(&mut self, idx: usize);

    fn add_neighborhood_score_mut(&mut self, pos: Pos);

    fn remove_neighborhood_score_mut(&mut self, pos: Pos);

}

impl ScoreTableOps for ScoreTable {
    fn set_slice_mut<const C: Color>(&mut self, idx: usize, threes: u8, fours: u8) {
        self.slice_pattern_counts.player_ref_mut::<C>()[idx].threes = threes;
        self.slice_pattern_counts.player_ref_mut::<C>()[idx].fours = fours;
    }

    fn clear_slice_mut<const C: Color>(&mut self, idx: usize) {
        self.slice_pattern_counts.player_ref_mut::<C>()[idx].threes = 0;
        self.slice_pattern_counts.player_ref_mut::<C>()[idx].fours = 0;
    }

    fn sum_slices<const C: Color>(&self) -> SlicePatternCount {
        let mut acc = SlicePatternCount::EMPTY;

        let mut entries_ptr = self.slice_pattern_counts.player_ref::<C>().as_ptr() as *const u8;

        // 72 % 8 = 0
        let mut vector_acc = Simd::splat(0);
        for start_idx in (0 .. slice::TOTAL_SLICE_AMOUNT).step_by(8) {
            vector_acc += Simd::<u8, 8>::from_slice(
                unsafe { std::slice::from_raw_parts(entries_ptr.add(start_idx), 8) }
            );
        }

        let result = vector_acc.to_array();

        for idx in 0 .. 4 {
            acc.threes += result[idx * 2];
            acc.fours += result[idx * 2 + 1];
        }

        acc
    }

    fn update_position_mut<const C: Color>(&mut self, idx: usize, pattern: Pattern) {
        self.position_scores.player_ref_mut::<C>()[idx] =
            PATTERN_SCORE_LUT.player_ref::<C>()[encode_pattern_to_score_key(pattern)]
    }

    fn clear_position_mut<const C: Color>(&mut self, idx: usize) {
        self.position_scores.player_ref_mut::<C>()[idx] = 0;
    }

    fn add_neighborhood_score_mut(&mut self, pos: Pos) {
        todo!()
    }

    fn remove_neighborhood_score_mut(&mut self, pos: Pos) {
        todo!()
    }
}

struct PassScoreTableOps;

impl ScoreTableOps for PassScoreTableOps {
    fn set_slice_mut<const C: Color>(&mut self, _idx: usize, _threes: u8, _fours: u8) {}

    fn clear_slice_mut<const C: Color>(&mut self, _idx: usize) {}

    fn sum_slices<const C: Color>(&self) -> SlicePatternCount {
        SlicePatternCount::EMPTY
    }

    fn update_position_mut<const C: Color>(&mut self, _idx: usize, _pattern: Pattern) {}

    fn clear_position_mut<const C: Color>(&mut self, _idx: usize) {}

    fn add_neighborhood_score_mut(&mut self, _pos: Pos) {}

    fn remove_neighborhood_score_mut(&mut self, _pos: Pos) {}
}

fn encode_pattern_to_score_key(pattern: Pattern) -> usize {
    let mut pattern_key = pattern.count_closed_fours() & 0b11; // 3 instructions

    pattern_key |= (pattern.count_open_fours() & 0b11) << 2; // 4 instructions
    pattern_key |= (pattern.count_open_threes() & 0b11) << 4; // 4 instructions
    pattern_key |= (pattern.has_five() as u32) << 6; // 4 instructions
    pattern_key |= (pattern.has_overline() as u32) << 7; // 4 instructions

    pattern_key as usize // 19 instructions
}

struct HeuristicScores; impl HeuristicScores {
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
                        HeuristicScores::FIVE
                    } else if has_overline > 0 {
                        HeuristicScores::OVERLINE_FORBID
                    } else if closed_fours + open_fours > 1 {
                        HeuristicScores::DOUBLE_FOUR_FORK
                    } else if open_threes > 1 {
                        HeuristicScores::DOUBLE_THREE_FORBID
                    } else if open_fours == 1 {
                        HeuristicScores::OPEN_FOUR
                    } else if closed_fours == 1 && open_threes == 1 {
                        HeuristicScores::THREE_FOUR_FORK
                    } else if open_threes == 1 {
                        HeuristicScores::OPEN_THREE
                    } else {
                        0
                    }
                },
                Color::White => {
                    if has_five != 0 {
                        HeuristicScores::FIVE
                    } else if open_fours > 0 {
                        HeuristicScores::OPEN_FOUR
                    } else if closed_fours > 1 {
                        HeuristicScores::DOUBLE_FOUR_FORK
                    } else if closed_fours > 0 && open_threes > 0 {
                        HeuristicScores::THREE_FOUR_FORK
                    } else if open_threes > 1 {
                        HeuristicScores::DOUBLE_THREE_FORK
                    } else if open_threes == 1 {
                        HeuristicScores::OPEN_THREE
                    } else if closed_fours == 1 {
                        HeuristicScores::CLOSED_FOUR
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
