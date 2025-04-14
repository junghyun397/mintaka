use crate::notation::color::{Color, ColorContainer};
use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::slice;
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
    pub position_scores: ColorContainer<[i8; pos::BOARD_SIZE]>
}

pub trait ScoreTableOps {

    fn set_slice_mut<const C: Color>(&mut self, idx: usize, threes: u8, fours: u8);

    fn clear_slice_mut<const C: Color>(&mut self, idx: usize);

    fn sum_slices<const C: Color>(&self) -> SlicePatternCount;

    fn add_neighborhood_score_mut(&mut self, pos: Pos);

    fn remove_neighborhood_score_mut(&mut self, pos: Pos);

}

impl ScoreTableOps for ScoreTable {
    fn set_slice_mut<const C: Color>(&mut self, idx: usize, threes: u8, fours: u8) {
        self.slice_pattern_counts.player_unit_mut::<C>().set_mut(idx, threes, fours);
    }

    fn clear_slice_mut<const C: Color>(&mut self, idx: usize) {
        self.slice_pattern_counts.player_unit_mut::<C>().clear_mut(idx);
    }

    fn sum_slices<const C: Color>(&self) -> SlicePatternCount {
        let mut acc = SlicePatternCount::EMPTY;

        let mut entries_ptr = self.slice_pattern_counts.player_unit::<C>().as_ptr() as *const u8;

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

    fn add_neighborhood_score_mut(&mut self, _pos: Pos) {}

    fn remove_neighborhood_score_mut(&mut self, _pos: Pos) {}
}
