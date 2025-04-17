use crate::notation::color::{AlignedColorContainer, Color, ColorContainer};
use crate::notation::direction::Direction;
use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::pattern::Pattern;
use crate::{const_for, slice};
use std::ops::{AddAssign, SubAssign};

#[derive(Debug, Copy, Clone)]
pub struct SlicePatternCount {
    pub threes: u8,
    pub closed_fours: u8,
    pub open_fours: u8,
    pub padding: u8,
}

impl SlicePatternCount {

    pub const EMPTY: Self = Self {
        threes: 0,
        closed_fours: 0,
        open_fours: 0,
        padding: 0,
    };

    pub fn total_fours(&self) -> u8 {
        self.closed_fours + self.open_fours
    }

}

impl SubAssign for SlicePatternCount {
    fn sub_assign(&mut self, rhs: Self) {
        self.threes -= rhs.threes;
        self.closed_fours -= rhs.closed_fours;
        self.open_fours -= rhs.open_fours;
    }
}

impl AddAssign for SlicePatternCount {
    fn add_assign(&mut self, rhs: Self) {
        self.threes += rhs.threes;
        self.closed_fours += rhs.closed_fours;
        self.open_fours += rhs.open_fours;
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SlicePatternCounts(pub [SlicePatternCount; slice::TOTAL_SLICE_AMOUNT]);

impl SlicePatternCounts {

    pub const EMPTY: Self = Self([SlicePatternCount::EMPTY; slice::TOTAL_SLICE_AMOUNT]);

    pub fn access_mut<const D: Direction>(&mut self, slice_idx: usize) -> &mut SlicePatternCount {
        &mut self.0[match D {
            Direction::Horizontal => 0,
            Direction::Vertical => pos::U_BOARD_WIDTH,
            Direction::Ascending => pos::U_BOARD_WIDTH * 2,
            Direction::Descending => pos::U_BOARD_WIDTH * 2 + slice::DIAGONAL_SLICE_AMOUNT,
        } + slice_idx]
    }

}

#[derive(Debug, Copy, Clone)]
pub struct ScoreTable {
    slice_pattern_counts: AlignedColorContainer<SlicePatternCounts>,
    pub position_scores: AlignedColorContainer<[i8; pos::BOARD_SIZE]>,
    pub slice_pattern_count: ColorContainer<SlicePatternCount>,
}

impl Default for ScoreTable {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl ScoreTable {

    pub const EMPTY: Self = Self {
        slice_pattern_counts: AlignedColorContainer::new(SlicePatternCounts::EMPTY, SlicePatternCounts::EMPTY),
        position_scores: AlignedColorContainer::new([0; pos::BOARD_SIZE], [0; pos::BOARD_SIZE]),
        slice_pattern_count: ColorContainer::new(SlicePatternCount::EMPTY, SlicePatternCount::EMPTY),
    };

}

impl ScoreTable {

    pub fn set_slice_mut<const C: Color, const D: Direction>(
        &mut self, slice_idx: usize, threes: u8, closed_fours: u8, open_fours: u8
    ) {
        let global_count = self.slice_pattern_count.player_ref_mut::<C>();
        let slice_count = self.slice_pattern_counts.player_ref_mut::<C>().access_mut::<D>(slice_idx);

        *global_count -= *slice_count;

        *slice_count = SlicePatternCount {
            threes,
            closed_fours,
            open_fours,
            padding: 0,
        };

        *global_count += *slice_count;
    }

    pub fn clear_slice_mut<const C: Color, const D: Direction>(&mut self, slice_idx: usize) {
        let global_count = self.slice_pattern_count.player_ref_mut::<C>();
        let slice_count = self.slice_pattern_counts.player_ref_mut::<C>().access_mut::<D>(slice_idx);

        *global_count -= *slice_count;

        *slice_count = SlicePatternCount::EMPTY;
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
                        HeuristicScores::DOUBLE_FOUR_FORBID
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
