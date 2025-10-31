use crate::bitfield::Bitfield;
use crate::notation::color::{AlignedColorContainer, Color, ColorContainer};
use crate::notation::direction::Direction;
use crate::notation::pos::Pos;
use crate::notation::rule::{ForbiddenKind, RuleKind};
use crate::slice::Slice;
use crate::slice_pattern::SlicePattern;
use crate::slice_pattern_count::SlicePatternCounts;
use crate::step_idx;
use crate::utils::lang_utils::{repeat_16x, repeat_4x};
use std::simd::cmp::SimdPartialEq;
use std::simd::Simd;

pub const CLOSED_FOUR_SINGLE: u8        = 0b1000_0000;
pub const CLOSED_FOUR_DOUBLE: u8        = 0b1100_0000;
pub const OPEN_FOUR: u8                 = 0b0010_0000;
pub const ANY_FOUR: u8                  = 0b1110_0000;
pub const FIVE: u8                      = 0b0001_0000;

pub const OPEN_THREE: u8                = 0b0000_1000;
pub const CLOSE_THREE: u8               = 0b0000_0100;
pub const OVERLINE: u8                  = 0b0000_0010;
pub const POTENTIAL: u8                 = 0b0000_0001;

pub const UNIT_CLOSED_FOUR_SINGLE_MASK: u32 = repeat_4x(CLOSED_FOUR_SINGLE);
pub const UNIT_CLOSED_FOUR_MASK: u32        = repeat_4x(CLOSED_FOUR_DOUBLE);
pub const UNIT_OPEN_FOUR_MASK: u32          = repeat_4x(OPEN_FOUR);
pub const UNIT_ANY_FOUR_MASK: u32           = repeat_4x(ANY_FOUR);
pub const UNIT_FIVE_MASK: u32               = repeat_4x(FIVE);

pub const UNIT_OPEN_THREE_MASK: u32         = repeat_4x(OPEN_THREE);
pub const UNIT_CLOSE_THREE_MASK: u32        = repeat_4x(CLOSE_THREE);
pub const UNIT_OVERLINE_MASK: u32           = repeat_4x(OVERLINE);
pub const UNIT_POTENTIAL_MASK: u32          = repeat_4x(POTENTIAL);

pub const UNIT_TACTICAL_MASK: u32           = repeat_4x(OPEN_THREE | ANY_FOUR);

pub const SLICE_PATTERN_CLOSED_FOUR_MASK: u128  = repeat_16x(CLOSED_FOUR_DOUBLE);
pub const SLICE_PATTERN_OPEN_FOUR_MASK: u128    = repeat_16x(OPEN_FOUR);
pub const SLICE_PATTERN_FIVE_MASK: u128         = repeat_16x(FIVE);
pub const SLICE_PATTERN_THREE_MASK: u128        = repeat_16x(OPEN_THREE);
pub const SLICE_PATTERN_OVERLINE_MASK: u128     = repeat_16x(OVERLINE);
pub const SLICE_PATTERN_MASK: u128              = repeat_16x(OVERLINE | ANY_FOUR | OPEN_THREE);

pub const PATTERN_SIZE: usize = 256;

#[derive(Eq, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum PatternCount {
    Cold = 0,
    Single = 1,
    Multiple = 2
}

impl PatternCount {

    fn from_masked_unit(masked: u32) -> Self {
        unsafe { std::mem::transmute::<u8, PatternCount>(masked.count_ones().min(2) as u8) }
    }

}

#[derive(Debug, Copy, Clone, Default)]
#[repr(C)]
pub struct Pattern {
    horizontal: u8,
    vertical: u8,
    ascending: u8,
    descending: u8
}

impl From<Pattern> for u32 {

    fn from(value: Pattern) -> Self {
        unsafe { std::mem::transmute::<Pattern, u32>(value) }
    }

}

impl Pattern {

    pub fn is_empty(&self) -> bool {
        u32::from(*self) == 0
    }

    pub fn is_tactical(&self) -> bool {
        self.apply_mask(UNIT_TACTICAL_MASK) != 0
    }

    pub fn has_three(&self) -> bool {
        self.apply_mask(UNIT_OPEN_THREE_MASK) != 0
    }

    pub fn has_threes(&self) -> bool {
        self.apply_mask(UNIT_OPEN_THREE_MASK).count_ones() > 1
    }

    pub fn has_any_four(&self) -> bool {
        self.apply_mask(UNIT_ANY_FOUR_MASK) != 0
    }

    pub fn has_open_four(&self) -> bool {
        self.apply_mask(UNIT_OPEN_FOUR_MASK) != 0
    }

    pub fn has_fours(&self) -> bool {
        self.apply_mask(UNIT_ANY_FOUR_MASK).count_ones() > 1
    }

    pub fn has_close_three(&self) -> bool {
        self.apply_mask(UNIT_CLOSE_THREE_MASK) != 0
    }

    pub fn has_five(&self) -> bool {
        self.apply_mask(UNIT_FIVE_MASK) != 0
    }

    pub fn has_overline(&self) -> bool {
        self.apply_mask(UNIT_OVERLINE_MASK) != 0
    }

    pub fn count_threes(&self) -> PatternCount {
        PatternCount::from_masked_unit(self.apply_mask(UNIT_OPEN_THREE_MASK))
    }

    pub fn count_fours(&self) -> PatternCount {
        PatternCount::from_masked_unit(self.apply_mask(UNIT_ANY_FOUR_MASK))
    }

    pub fn count_open_threes(&self) -> u32 {
        self.apply_mask(UNIT_OPEN_THREE_MASK).count_ones()
    }

    pub fn count_close_threes(&self) -> u32 {
        self.apply_mask(UNIT_CLOSE_THREE_MASK).count_ones()
    }

    pub fn count_closed_fours(&self) -> u32 {
        self.apply_mask(UNIT_CLOSED_FOUR_MASK).count_ones()
    }

    pub fn count_open_fours(&self) -> u32 {
        self.apply_mask(UNIT_OPEN_FOUR_MASK).count_ones()
    }

    pub fn count_total_fours(&self) -> u32 {
        self.apply_mask(UNIT_ANY_FOUR_MASK).count_ones()
    }

    pub fn count_fives(&self) -> u32 {
        self.apply_mask(UNIT_FIVE_MASK).count_ones()
    }

    pub fn count_potentials(&self) -> u32 {
        self.apply_mask(UNIT_POTENTIAL_MASK).count_ones()
    }

    pub fn iter_three_directions(&self) -> impl Iterator<Item=Direction> + '_ {
        DirectionIterator { packed_unit: self.apply_mask(UNIT_OPEN_THREE_MASK) }
    }

    pub fn iter_four_directions(&self) -> impl Iterator<Item=Direction> + '_ {
        DirectionIterator { packed_unit: self.apply_mask(UNIT_ANY_FOUR_MASK) }
    }

    pub fn has_invalid_double_three(&self) -> bool {
        self.descending & POTENTIAL == POTENTIAL
    }

    pub fn mark_invalid_double_three(&mut self) {
        self.descending |= POTENTIAL;
    }

    pub fn unmark_invalid_double_three(&mut self) {
        self.descending &= !POTENTIAL;
    }

    pub fn has_fork_unchecked(&self) -> bool {
        self.apply_mask(UNIT_CLOSED_FOUR_MASK | UNIT_OPEN_THREE_MASK).count_ones() > 1
    }

    pub fn is_forbidden_unchecked(&self) -> bool {
        (self.count_total_fours() > 1 || self.count_open_threes() > 1 || self.has_overline())
            && !self.has_five()
    }

    pub fn apply_mask_mut<const D: Direction>(&mut self, pattern: u8) {
        match D {
            Direction::Horizontal => self.horizontal = pattern,
            Direction::Vertical => self.vertical = pattern,
            Direction::Ascending => self.ascending = pattern,
            Direction::Descending => self.descending = pattern
        }
    }

    pub fn apply_mask(&self, mask: u32) -> u32 {
        u32::from(*self) & mask
    }

}

#[derive(Debug, Copy, Clone)]
pub struct Patterns {
    pub field: AlignedColorContainer<[Pattern; PATTERN_SIZE]>,
    pub counts: SlicePatternCounts,
    pub unchecked_five_pos: ColorContainer<Option<Pos>>,
    pub candidate_forbidden_field: Bitfield,
    pub forbidden_field: Bitfield,
}

impl Default for Patterns {

    fn default() -> Self {
        Self {
            field: unsafe { std::mem::zeroed() },
            counts: SlicePatternCounts::EMPTY,
            unchecked_five_pos: ColorContainer::new(None, None),
            candidate_forbidden_field: Bitfield::ZERO_FILLED,
            forbidden_field: Bitfield::ZERO_FILLED,
        }
    }

}

impl Patterns {

    pub const EMPTY_UNCHECKED_FIVE_POS: ColorContainer<Option<Pos>> = ColorContainer::new(None, None);

    pub fn is_forbidden(&self, pos: Pos) -> bool {
        self.forbidden_field.is_hot(pos)
    }

    pub fn forbidden_kind(&self, pos: Pos) -> Option<ForbiddenKind> {
        self.is_forbidden(pos).then(|| {
            let pattern = self.field.get::<{ Color::Black }>()[pos.idx_usize()];

            if pattern.has_threes() {
                ForbiddenKind::DoubleThree
            } else if pattern.has_fours() {
                ForbiddenKind::DoubleFour
            } else {
                ForbiddenKind::Overline
            }
        })
    }

    pub fn update_with_slice_mut<const R: RuleKind, const C: Color, const D: Direction>(&mut self, slice: &mut Slice) {
        let slice_pattern = slice.calculate_slice_pattern::<R, C>();

        match (slice.pattern_bitmap.get::<C>() == 0, slice_pattern.is_empty()) {
            (false, true) =>
                self.clear_with_slice_mut::<C, D>(slice),
            (_, false) =>
                self.update_with_slice_pattern_mut::<C, D>(slice, slice_pattern),
            _ => {}
        };
    }

    pub fn clear_with_slice_mut<const C: Color, const D: Direction>(&mut self, slice: &mut Slice) {
        self.counts.clear_slice_mut::<C, D>(slice.idx as usize);

        let start_idx = slice.start_pos.idx_usize();

        let mut clear_mask = std::mem::take(slice.pattern_bitmap.get_ref_mut::<C>());

        while clear_mask != 0 {
            let slice_idx = clear_mask.trailing_zeros() as usize;
            clear_mask &= clear_mask - 1;

            let idx = step_idx!(D, start_idx, slice_idx);
            self.field.get_ref_mut::<C>()[idx].apply_mask_mut::<D>(0);
        }
    }

    fn update_with_slice_pattern_mut<const C: Color, const D: Direction>(
        &mut self, slice: &mut Slice, slice_pattern: SlicePattern
    ) {
        *self.unchecked_five_pos.get_ref_mut::<C>() = (slice_pattern.patterns & SLICE_PATTERN_FIVE_MASK != 0)
            .then(|| {
                let slice_idx = (slice_pattern.patterns & SLICE_PATTERN_FIVE_MASK).trailing_zeros() / 8;
                Pos::from_index(step_idx!(D, slice.start_pos.idx(), slice_idx as u8))
            })
            .or(self.unchecked_five_pos.get::<C>());

        self.counts.update_slice_mut::<C, D>(
            slice.idx as usize,
            (slice_pattern.patterns & SLICE_PATTERN_THREE_MASK).count_ones() as u8,
            (slice_pattern.patterns & SLICE_PATTERN_CLOSED_FOUR_MASK).count_ones() as u8,
            (slice_pattern.patterns & SLICE_PATTERN_OPEN_FOUR_MASK).count_ones() as u8,
        );

        let pattern_bitmap = std::mem::replace(
            slice.pattern_bitmap.get_ref_mut::<C>(),
            encode_u128_into_u16(slice_pattern.patterns)
        );

        let slice_patterns = slice_pattern.patterns.to_le_bytes();

        let start_idx = slice.start_pos.idx_usize();
        let mut update_mask = slice.pattern_bitmap.get::<C>() | pattern_bitmap;
        while update_mask != 0 {
            let slice_idx = update_mask.trailing_zeros() as usize;
            update_mask &= update_mask - 1;

            let idx = step_idx!(D, start_idx, slice_idx);
            self.field.get_ref_mut::<C>()[idx].apply_mask_mut::<D>(slice_patterns[slice_idx]);

            if C == Color::Black && self.field[Color::Black][idx].is_forbidden_unchecked() {
                self.candidate_forbidden_field.set_idx(idx);
            }
        }
    }

}

struct DirectionIterator {
    packed_unit: u32
}

impl Iterator for DirectionIterator {

    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        (self.packed_unit != 0).then(|| {
            let tails = self.packed_unit.trailing_zeros();
            self.packed_unit &= self.packed_unit - 1;

            Direction::from(tails as u8 / 8)
        })
    }

}

fn encode_u128_into_u16(source: u128) -> u16 {
    Simd::<u8, 16>::from(source.to_le_bytes())
        .simd_ne(Simd::splat(0))
        .to_bitmask() as u16
}
