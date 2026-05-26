use crate::bitfield::Bitfield;
use crate::notation::color::Color;
use crate::notation::direction::Direction;
use crate::pattern::Pattern;
use crate::{pattern, slice};
use crate::step_idx;
use crate::utils::empty::Empty;
use std::simd::cmp::SimdPartialEq;
use std::simd::{u8x16, Simd};
use crate::notation::pos;

#[derive(Debug, Copy, Clone)]
#[repr(align(32))]
pub struct PatternIndex {
    pub open_threes: Bitfield,
    pub close_threes: Bitfield,
    pub fork_fours: Bitfield,
    pub closed_fours: Bitfield,
    pub slice_bitmap: [SliceBitmap; slice::TOTAL_SLICE_AMOUNT],
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C, packed)]
pub struct SliceBitmap {
    open_threes: u16,
    close_threes: u16,
    open_fours: u16,
    closed_fours: u16,
}

impl Empty for SliceBitmap {
    fn empty() -> Self {
        Self {
            open_threes: 0,
            close_threes: 0,
            open_fours: 0,
            closed_fours: 0,
        }
    }
}

impl Empty for PatternIndex {
    fn empty() -> Self {
        Self {
            open_threes: Bitfield::empty(),
            close_threes: Bitfield::empty(),
            fork_fours: Bitfield::empty(),
            closed_fours: Bitfield::empty(),
            slice_bitmap: [SliceBitmap::empty(); slice::TOTAL_SLICE_AMOUNT],
        }
    }
}

impl PatternIndex {
    #[inline(always)]
    pub fn has_any_four(&self) -> bool {
        !(self.closed_fours.is_empty() && self.fork_fours.is_empty())
    }

    #[inline(always)]
    pub fn three_four_forks(&self) -> Bitfield {
        self.open_threes & self.closed_fours
    }

    #[inline(always)]
    pub fn any_fours(&self) -> Bitfield {
        self.fork_fours | self.closed_fours
    }

    #[inline(always)]
    pub fn slice_bitmap<const D: Direction>(&self, slice_idx: u8) -> SliceBitmap {
        self.slice_bitmap[Self::local_slice_idx::<D>(slice_idx as usize)]
    }

    #[inline(always)]
    pub fn replace_slice_bitmap<const D: Direction>(
        &mut self,
        slice_idx: u8,
        bitmap: SliceBitmap,
    ) -> SliceBitmap {
        std::mem::replace(
            &mut self.slice_bitmap[Self::local_slice_idx::<D>(slice_idx as usize)],
            bitmap,
        )
    }

    #[inline(always)]
    pub fn update_slice_bitfields<const C: Color, const D: Direction>(
        &mut self,
        pattern_field: &[Pattern; pattern::PATTERN_SIZE],
        start_idx: usize,
        old_bitmap: SliceBitmap,
        new_bitmap: SliceBitmap,
    ) {
        if old_bitmap == new_bitmap {
            return;
        }

        Self::update_slice_bitfield::<D>(
            pattern_field, &mut self.open_threes, start_idx,
            old_bitmap.open_threes, new_bitmap.open_threes,
            Pattern::has_three,
        );

        Self::update_slice_bitfield::<D>(
            pattern_field, &mut self.close_threes, start_idx,
            old_bitmap.close_threes, new_bitmap.close_threes,
            Pattern::has_close_three,
        );

        Self::update_slice_bitfield::<D>(
            pattern_field, &mut self.closed_fours, start_idx,
            old_bitmap.closed_fours, new_bitmap.closed_fours,
            Pattern::has_closed_four,
        );

        if C == Color::Black {
            Self::update_slice_bitfield::<D>(
                pattern_field, &mut self.fork_fours, start_idx,
                old_bitmap.open_fours, new_bitmap.open_fours,
                Pattern::has_open_four,
            );
        } else {
            self.update_fork_four_slice_bitfield::<C, D>(
                pattern_field,
                start_idx,
                (old_bitmap.open_fours ^ new_bitmap.open_fours)
                    | old_bitmap.closed_fours
                    | new_bitmap.closed_fours,
            );
        }
    }

    #[inline(always)]
    fn local_slice_idx<const D: Direction>(slice_idx: usize) -> usize {
        (match D {
            Direction::Horizontal => 0,
            Direction::Vertical => pos::U_BOARD_WIDTH,
            Direction::Ascending => pos::U_BOARD_WIDTH * 2,
            Direction::Descending => pos::U_BOARD_WIDTH * 2 + slice::DIAGONAL_SLICE_AMOUNT,
        }) + slice_idx
    }

    #[inline(always)]
    fn update_slice_bitfield<const D: Direction>(
        pattern_field: &[Pattern; pattern::PATTERN_SIZE],
        bitfield: &mut Bitfield,
        start_idx: usize,
        old_bitmask: u16,
        new_bitmask: u16,
        is_present: impl Fn(&Pattern) -> bool,
    ) {
        if old_bitmask == new_bitmask {
            return;
        }

        let mut added = new_bitmask & !old_bitmask;
        while added != 0 {
            let slice_idx = added.trailing_zeros() as usize;
            added &= added - 1;

            bitfield.set_idx(step_idx!(D, start_idx, slice_idx));
        }

        let mut removed = old_bitmask & !new_bitmask;
        while removed != 0 {
            let slice_idx = removed.trailing_zeros() as usize;
            removed &= removed - 1;

            let idx = step_idx!(D, start_idx, slice_idx);

            if !is_present(&pattern_field[idx]) {
                bitfield.unset_idx(idx);
            }
        }
    }

    #[inline(always)]
    fn update_fork_four_slice_bitfield<const C: Color, const D: Direction>(
        &mut self,
        field: &[Pattern; pattern::PATTERN_SIZE],
        start_idx: usize,
        mut changed_bitmask: u16,
    ) {
        while changed_bitmask != 0 {
            let slice_idx = changed_bitmask.trailing_zeros() as usize;
            changed_bitmask &= changed_bitmask - 1;

            let idx = step_idx!(D, start_idx, slice_idx);

            let is_fork_four = if C == Color::Black {
                field[idx].has_open_four()
            } else {
                let closed_fours = field[idx].apply_mask(pattern::UNIT_CLOSED_FOUR_MASK);

                field[idx].has_open_four()
                    || (closed_fours != 0 && (closed_fours & (closed_fours - 1)) != 0)
            };

            self.fork_fours.set_bit_idx(
                idx,
                is_fork_four,
            );
        }
    }
}

#[inline(always)]
pub fn pattern_bitmaps_from_patterns(slice_pattern: [u8; 16]) -> (u16, SliceBitmap) {
    let slice_pattern = Simd::<u8, 16>::from(slice_pattern);

    let slice_bitmap = SliceBitmap {
        open_threes: pattern_bitmask(slice_pattern, pattern::OPEN_THREE),
        close_threes: pattern_bitmask(slice_pattern, pattern::CLOSE_THREE),
        open_fours: pattern_bitmask(slice_pattern, pattern::OPEN_FOUR),
        closed_fours: pattern_bitmask(slice_pattern, pattern::CLOSED_FOUR_SINGLE),
    };

    let slice_pattern_bitmap = pattern_nonzero_bitmask(slice_pattern);

    (slice_pattern_bitmap, slice_bitmap)
}

#[inline(always)]
fn pattern_nonzero_bitmask(patterns: u8x16) -> u16 {
    patterns
        .simd_ne(Simd::splat(0))
        .to_bitmask() as u16
}

#[inline(always)]
fn pattern_bitmask(patterns: u8x16, mask: u8) -> u16 {
    (patterns & Simd::splat(mask))
        .simd_ne(Simd::splat(0))
        .to_bitmask() as u16
}
