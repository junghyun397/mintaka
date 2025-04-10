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
#[repr(align(32))]
pub struct SlicePatternCounts {
    pub entries: [SlicePatternCount; slice::TOTAL_SLICE_AMOUNT],
}

impl SlicePatternCounts {

    pub fn set_mut(&mut self, idx: usize, threes: u8, fours: u8) {
        self.entries[idx].threes = threes;
        self.entries[idx].fours = fours;
    }

    pub fn clear_mut(&mut self, idx: usize) {
        self.entries[idx].threes = 0;
        self.entries[idx].fours = 0;
    }

    pub fn sum(&self) -> SlicePatternCount {
        let mut acc = SlicePatternCount::EMPTY;

        let mut entries_ptr = self.entries.as_ptr() as *const u8;

        let mut vector_acc = Simd::splat(0);

        // 72 % 8 = 0
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
}
