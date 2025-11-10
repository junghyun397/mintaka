use crate::notation::color::{Color, ColorContainer};
use crate::notation::direction::Direction;
use crate::notation::pos;
use crate::slice;
use std::simd::cmp::SimdPartialEq;
use std::simd::Simd;

pub const PADDED_SLICE_AMOUNT: usize = (slice::TOTAL_SLICE_AMOUNT | 31) + 1;

#[derive(Debug, Copy, Clone)]
pub struct SlicePatternCount {
    pub threes: [u8; PADDED_SLICE_AMOUNT],
    pub closed_fours: [u8; PADDED_SLICE_AMOUNT],
    pub open_fours: [u8; PADDED_SLICE_AMOUNT],
}

impl SlicePatternCount {

    pub fn total_open_four_structs_unchecked(&self) -> u32 {
        let ptr = self.open_fours.as_ptr();

        let zero_mask = Simd::splat(0);

        let mut total_components_unchecked = 0;

        for start_idx in (0 .. PADDED_SLICE_AMOUNT).step_by(32) {
            let vector = Simd::<u8, 32>::from_slice(
                unsafe { std::slice::from_raw_parts(ptr.add(start_idx), 32) }
            );

            total_components_unchecked += vector
                .simd_ne(zero_mask)
                .to_bitmask()
                .count_ones();
        }

        total_components_unchecked
    }

}

#[derive(Debug, Copy, Clone)]
pub struct GlobalPatternCount {
    pub threes: u8,
    pub closed_fours: u8,
    pub open_fours: u8,
}

impl GlobalPatternCount {

    pub fn total_fours(&self) -> u8 {
        self.closed_fours + self.open_fours
    }

}

#[derive(Debug, Copy, Clone)]
pub struct SlicePatternCounts {
    pub slice: ColorContainer<SlicePatternCount>,
    pub global: ColorContainer<GlobalPatternCount>,
}

impl SlicePatternCounts {

    pub const EMPTY: Self = unsafe { std::mem::zeroed() };

    fn calculate_local_slice_idx<const D: Direction>(slice_idx: usize) -> usize {
        (match D {
            Direction::Horizontal => 0,
            Direction::Vertical => pos::U_BOARD_WIDTH,
            Direction::Ascending => pos::U_BOARD_WIDTH * 2,
            Direction::Descending => pos::U_BOARD_WIDTH * 2 + slice::DIAGONAL_SLICE_AMOUNT,
        }) + slice_idx
    }

    pub fn update_slice_mut<const C: Color, const D: Direction>(
        &mut self, slice_idx: usize, threes: u8, closed_fours: u8, open_fours: u8
    ) {
        let local_slice_idx = Self::calculate_local_slice_idx::<D>(slice_idx);

        self.global[C].threes += threes;
        self.global[C].closed_fours += closed_fours;
        self.global[C].open_fours += open_fours;

        self.global[C].threes -= std::mem::replace(&mut self.slice[C].threes[local_slice_idx], threes);
        self.global[C].closed_fours -= std::mem::replace(&mut self.slice[C].closed_fours[local_slice_idx], closed_fours);
        self.global[C].open_fours -= std::mem::replace(&mut self.slice[C].open_fours[local_slice_idx], open_fours);
    }

    pub fn clear_slice_mut<const C: Color, const D: Direction>(&mut self, slice_idx: usize) {
        let local_slice_idx = Self::calculate_local_slice_idx::<D>(slice_idx);

        let threes = std::mem::take(&mut self.slice[C].threes[local_slice_idx]);
        let closed_fours = std::mem::take(&mut self.slice[C].closed_fours[local_slice_idx]);
        let open_fours = std::mem::take(&mut self.slice[C].open_fours[local_slice_idx]);

        self.global[C].threes -= threes;
        self.global[C].closed_fours -= closed_fours;
        self.global[C].open_fours -= open_fours;
    }

}
