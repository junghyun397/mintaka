use crate::notation::color::{Color, ColorContainer};
use crate::notation::direction::Direction;
use crate::notation::pos;
use crate::slice;

#[derive(Debug, Copy, Clone)]
pub struct SlicePatternCount {
    pub threes: u8,
    pub closed_fours: u8,
    pub open_fours: u8,
    pub score: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct GlobalPatternCount {
    pub threes: i16,
    pub closed_fours: i16,
    pub open_fours: i16,
    pub score: i16,
}

impl SlicePatternCount {

    pub const EMPTY: Self = Self {
        threes: 0,
        closed_fours: 0,
        open_fours: 0,
        score: 0,
    };

    pub fn total_fours(&self) -> u8 {
        self.closed_fours + self.open_fours
    }

}

#[derive(Debug, Copy, Clone)]
pub struct SlicePatternCounts {
    locals: ColorContainer<[SlicePatternCount; slice::TOTAL_SLICE_AMOUNT]>,
    pub global: ColorContainer<GlobalPatternCount>,
}

impl SlicePatternCounts {

    pub const EMPTY: Self = unsafe { std::mem::zeroed() };

    fn access_local_mut<const C: Color, const D: Direction>(locals: &mut ColorContainer<[SlicePatternCount; slice::TOTAL_SLICE_AMOUNT]>, slice_idx: usize) -> &mut SlicePatternCount {
        &mut locals.get_ref_mut::<C>()[match D {
            Direction::Horizontal => 0,
            Direction::Vertical => pos::U_BOARD_WIDTH,
            Direction::Ascending => pos::U_BOARD_WIDTH * 2,
            Direction::Descending => pos::U_BOARD_WIDTH * 2 + slice::DIAGONAL_SLICE_AMOUNT,
        } + slice_idx]
    }

    pub fn set_slice_mut<const C: Color, const D: Direction>(
        &mut self, slice_idx: usize, threes: u8, closed_fours: u8, open_fours: u8, score: i16,
    ) {
        let global_count = self.global.get_ref_mut::<C>();
        let slice_count = Self::access_local_mut::<C, D>(&mut self.locals, slice_idx);

        global_count.threes -= slice_count.threes as i16;
        global_count.closed_fours -= slice_count.closed_fours as i16;
        global_count.open_fours -= slice_count.open_fours as i16;
        global_count.score -= slice_count.score as i16;

        global_count.threes += threes as i16;
        global_count.closed_fours += closed_fours as i16;
        global_count.open_fours += open_fours as i16;
        global_count.score += score;

        *slice_count = SlicePatternCount {
            threes,
            closed_fours,
            open_fours,
            score: score as u8,
        };

    }

    pub fn clear_slice_mut<const C: Color, const D: Direction>(&mut self, slice_idx: usize, score: i16) {
        let global_count = self.global.get_ref_mut::<C>();
        let slice_count = Self::access_local_mut::<C, D>(&mut self.locals, slice_idx);

        global_count.threes -= slice_count.threes as i16;
        global_count.closed_fours -= slice_count.closed_fours as i16;
        global_count.open_fours -= slice_count.open_fours as i16;
        global_count.score -= slice_count.score as i16;

        *slice_count = SlicePatternCount::EMPTY;
        slice_count.score = score as u8;
    }

    // TODO: performance optimization
    pub fn update_slice_score_mut<const C: Color, const D: Direction>(&mut self, slice_idx: usize, score: i16) {
        let global_count = self.global.get_ref_mut::<C>();
        let slice_count = Self::access_local_mut::<C, D>(&mut self.locals, slice_idx);

        global_count.score -= slice_count.score as i16;
        global_count.score += score;

        slice_count.score = score as u8;
    }

}
