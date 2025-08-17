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
    pub threes: u8,
    pub closed_fours: u8,
    pub open_fours: u8,
    pub score: u16,
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

    fn access_local_mut<const C: Color, const D: Direction>(
        locals: &mut ColorContainer<[SlicePatternCount; slice::TOTAL_SLICE_AMOUNT]>, slice_idx: usize
    ) -> &mut SlicePatternCount {
        &mut locals.get_ref_mut::<C>()[match D {
            Direction::Horizontal => 0,
            Direction::Vertical => pos::U_BOARD_WIDTH,
            Direction::Ascending => pos::U_BOARD_WIDTH * 2,
            Direction::Descending => pos::U_BOARD_WIDTH * 2 + slice::DIAGONAL_SLICE_AMOUNT,
        } + slice_idx]
    }

    pub fn update_slice_mut<const C: Color, const D: Direction>(
        &mut self, slice_idx: usize, threes: u8, closed_fours: u8, open_fours: u8, score: u8
    ) {
        let global_count = self.global.get_ref_mut::<C>();
        let slice_count = Self::access_local_mut::<C, D>(&mut self.locals, slice_idx);

        global_count.threes += threes;
        global_count.threes -= slice_count.threes;

        global_count.closed_fours += closed_fours;
        global_count.closed_fours -= slice_count.closed_fours;

        global_count.open_fours += open_fours;
        global_count.open_fours -= slice_count.open_fours;

        global_count.score += score as u16;
        global_count.score -= slice_count.score as u16;

        *slice_count = SlicePatternCount {
            threes,
            closed_fours,
            open_fours,
            score,
        };
    }

    pub fn clear_slice_mut<const C: Color, const D: Direction>(&mut self, slice_idx: usize, score: u8) {
        let global_count = self.global.get_ref_mut::<C>();
        let slice_count = Self::access_local_mut::<C, D>(&mut self.locals, slice_idx);

        global_count.threes -= slice_count.threes;
        global_count.closed_fours -= slice_count.closed_fours;
        global_count.open_fours -= slice_count.open_fours;

        global_count.score += score as u16;
        global_count.score -= slice_count.score as u16;

        *slice_count = SlicePatternCount {
            threes: 0,
            closed_fours: 0,
            open_fours: 0,
            score,
        };
    }

    pub fn update_slice_score_mut<const C: Color, const D: Direction>(&mut self, slice_idx: usize, score: u8) {
        let slice_count = Self::access_local_mut::<C, D>(&mut self.locals, slice_idx);

        self.global.get_ref_mut::<C>().score += score as u16;
        self.global.get_ref_mut::<C>().score -= slice_count.score as u16;

        slice_count.score = score;
    }

}

impl GlobalPatternCount {

    pub fn total_fours(&self) -> u8 {
        self.closed_fours + self.open_fours
    }

}
