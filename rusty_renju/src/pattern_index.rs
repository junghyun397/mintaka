use crate::bitfield::Bitfield;
use crate::notation::color::Color;
use crate::notation::direction::Direction;
use crate::pattern::Pattern;
use crate::{pattern, slice};
use crate::slice_pattern::SlicePattern;
use crate::step_idx;
use crate::utils::empty::Empty;
use std::simd::cmp::SimdPartialEq;
use std::simd::{u8x16, Simd};
use crate::notation::pos;
use crate::notation::rule::RuleKind;

#[derive(Debug, Copy, Clone)]
#[repr(align(32))]
pub struct PatternIndex {
    pub open_threes: Bitfield,
    pub close_threes: Bitfield,
    pub fork_fours: Bitfield,
    pub closed_fours: Bitfield,
    pub slice_bitmap: [SlicePattern; slice::TOTAL_SLICE_AMOUNT],
}

impl Empty for PatternIndex {
    fn empty() -> Self {
        Self {
            open_threes: Bitfield::empty(),
            close_threes: Bitfield::empty(),
            fork_fours: Bitfield::empty(),
            closed_fours: Bitfield::empty(),
            slice_bitmap: [SlicePattern::EMPTY; slice::TOTAL_SLICE_AMOUNT],
        }
    }
}

impl PatternIndex {
    #[inline(always)]
    pub fn has_any_four(&self) -> bool {
        !(self.closed_fours.is_empty() && self.fork_fours.is_empty())
    }

    #[inline(always)]
    pub fn replace_slice_bitmap<const D: Direction>(
        &mut self,
        slice_idx: u8,
        bitmap: SlicePattern,
    ) -> SlicePattern {
        std::mem::replace(
            &mut self.slice_bitmap[Self::local_slice_idx::<D>(slice_idx as usize)],
            bitmap,
        )
    }

    #[inline(always)]
    pub fn update_slice_bitfields<const R: RuleKind, const C: Color, const D: Direction>(
        &mut self,
        pattern_field: &[Pattern; pattern::PATTERN_SIZE],
        start_idx: usize,
        old_bitmap: SlicePattern,
        new_bitmap: SlicePattern,
    ) {
        if old_bitmap == new_bitmap {
            return;
        }

        let old_patterns = Simd::<u8, 16>::from(old_bitmap.patterns.to_le_bytes());
        let new_patterns = Simd::<u8, 16>::from(new_bitmap.patterns.to_le_bytes());

        macro_rules! update_slice_bitfield {
            ($bitfield:expr,$mask:expr,$is_present:ident) => {{
                let old_bitmask = pattern_bitmask(old_patterns, $mask);
                let new_bitmask = pattern_bitmask(new_patterns, $mask);

                if old_bitmask != new_bitmask {
                    let mut added = new_bitmask & !old_bitmask;
                    while added != 0 {
                        let slice_idx = added.trailing_zeros() as usize;
                        added &= added - 1;

                        $bitfield.set_idx(step_idx!(D, start_idx, slice_idx));
                    }

                    let mut removed = old_bitmask & !new_bitmask;
                    while removed != 0 {
                        let slice_idx = removed.trailing_zeros() as usize;
                        removed &= removed - 1;

                        let idx = step_idx!(D, start_idx, slice_idx);

                        if !pattern_field[idx].$is_present() {
                            $bitfield.unset_idx(idx);
                        }
                    }
                }
            }};
        }

        update_slice_bitfield!(
            self.open_threes,
            pattern::OPEN_THREE,
            has_open_three
        );

        update_slice_bitfield!(
            self.close_threes,
            pattern::CLOSE_THREE,
            has_close_three
        );

        update_slice_bitfield!(
            self.closed_fours,
            pattern::CLOSED_FOUR_SINGLE,
            has_closed_four
        );

        if R == RuleKind::Renju && C == Color::Black {
            let old_open_fours = pattern_bitmask(old_patterns, pattern::OPEN_FOUR);
            let new_open_fours = pattern_bitmask(new_patterns, pattern::OPEN_FOUR);
            let old_closed_fours = pattern_bitmask(old_patterns, pattern::CLOSED_FOUR_SINGLE);
            let new_closed_fours = pattern_bitmask(new_patterns, pattern::CLOSED_FOUR_SINGLE);
            let mut changed_bitmask = (old_open_fours ^ new_open_fours)
                | old_closed_fours
                | new_closed_fours;

            while changed_bitmask != 0 {
                let slice_idx = changed_bitmask.trailing_zeros() as usize;
                changed_bitmask &= changed_bitmask - 1;

                let idx = step_idx!(D, start_idx, slice_idx);

                let is_fork_four = match (R, C) {
                    (RuleKind::Renju, Color::Black) => pattern_field[idx].has_open_four(),
                    _ => pattern_field[idx].has_any_fours(),
                };

                self.fork_fours.set_bit_idx(
                    idx,
                    is_fork_four,
                );
            }
        } else {
            update_slice_bitfield!(
                self.fork_fours,
                pattern::OPEN_FOUR,
                has_open_four
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
}

#[inline(always)]
fn pattern_bitmask(patterns: u8x16, mask: u8) -> u16 {
    (patterns & Simd::splat(mask))
        .simd_ne(Simd::splat(0))
        .to_bitmask() as u16
}
