use crate::bitfield::Bitfield;
use crate::notation::color::{AlignedColorContainer, Color, ColorContainer};
use crate::notation::direction::Direction;
use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::notation::rule::ForbiddenKind;
use crate::score_table::ScoreTable;
use crate::slice::Slice;
use crate::slice_pattern::{contains_five_in_a_row, SlicePattern};
use crate::utils::lang_utils::{repeat_16x, repeat_4x};
use crate::{assert_struct_sizes, step_idx};
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
pub const MARKER: u8                    = 0b0000_0001;

pub const UNIT_CLOSED_FOUR_SINGLE_MASK: u32 = repeat_4x(CLOSED_FOUR_SINGLE);
pub const UNIT_CLOSED_FOUR_MASK: u32        = repeat_4x(CLOSED_FOUR_DOUBLE);
pub const UNIT_OPEN_FOUR_MASK: u32          = repeat_4x(OPEN_FOUR);
pub const UNIT_ANY_FOUR_MASK: u32           = repeat_4x(ANY_FOUR);
pub const UNIT_FIVE_MASK: u32               = repeat_4x(FIVE);

pub const UNIT_OPEN_THREE_MASK: u32         = repeat_4x(OPEN_THREE);
pub const UNIT_CLOSE_THREE_MASK: u32        = repeat_4x(CLOSE_THREE);
pub const UNIT_OVERLINE_MASK: u32           = repeat_4x(OVERLINE);

pub const UNIT_PATTERN_MASK: u32            = repeat_4x(!MARKER);

pub const SLICE_PATTERN_THREE_MASK: u128        = repeat_16x(OPEN_THREE);
pub const SLICE_PATTERN_CLOSED_FOUR_MASK: u128  = repeat_16x(CLOSED_FOUR_SINGLE);
pub const SLICE_PATTERN_OPEN_FOUR_MASK: u128    = repeat_16x(OPEN_FOUR);
pub const SLICE_PATTERN_FIVE_MASK: u128         = repeat_16x(FIVE);

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

    pub fn has_three_four_fork(&self) -> bool {
        self.apply_mask(UNIT_ANY_FOUR_MASK) != 0 && self.apply_mask(UNIT_OPEN_THREE_MASK) != 0
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

    pub fn iter_three_directions(&self) -> impl Iterator<Item=Direction> + '_ {
        DirectionIterator { packed_unit: self.apply_mask(UNIT_OPEN_THREE_MASK) }
    }

    pub fn iter_four_directions(&self) -> impl Iterator<Item=Direction> + '_ {
        DirectionIterator { packed_unit: self.apply_mask(UNIT_ANY_FOUR_MASK) }
    }

    pub fn has_invalid_double_three(&self) -> bool {
        self.descending & MARKER == MARKER
    }

    pub fn mark_invalid_double_three(&mut self) {
        self.descending |= MARKER;
    }

    pub fn unmark_invalid_double_three(&mut self) {
        self.descending &= !MARKER;
    }

    pub fn is_forbidden(&self) -> bool {
        !self.is_empty()
            && (self.has_fours()
                || (self.has_threes() && !self.has_invalid_double_three())
                || self.has_overline()
            )
            && !self.has_five()
    }

    pub fn forbidden_kind(&self) -> Option<ForbiddenKind> {
        if self.is_forbidden() {
            if self.has_threes() {
                Some(ForbiddenKind::DoubleThree)
            } else if self.has_fours() {
                Some(ForbiddenKind::DoubleFour)
            } else {
                Some(ForbiddenKind::Overline)
            }
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn apply_mask_mut<const C: Color, const D: Direction>(&mut self, pattern: u8) {
        match (C, D) {
            (_, Direction::Horizontal) => self.horizontal = pattern,
            (_, Direction::Vertical) => self.vertical = pattern,
            (_, Direction::Ascending) => self.ascending = pattern,
            (Color::Black, Direction::Descending) =>
                self.descending = pattern | (self.descending & MARKER), // retain invalid three makers
            (Color::White, Direction::Descending) => self.descending = pattern
        }
    }

    pub fn apply_mask(&self, mask: u32) -> u32 {
        u32::from(*self) & mask
    }

}

pub struct SlicePatternCount {
    pub threes: u8,
    pub fours: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct Patterns {
    pub field: AlignedColorContainer<[Pattern; pos::BOARD_SIZE]>,
    pub score_table: ScoreTable,
    pub unchecked_five_in_a_row: Option<Color>,
    pub unchecked_five_pos: ColorContainer<Option<Pos>>,
    pub unchecked_double_three_field: Bitfield,
}

assert_struct_sizes!(Patterns, size=3072, align=64);

impl Default for Patterns {

    fn default() -> Self {
        Self {
            field: unsafe { std::mem::zeroed() },
            score_table: ScoreTable::EMPTY,
            unchecked_five_in_a_row: None,
            unchecked_five_pos: ColorContainer {
                black: None,
                white: None
            },
            unchecked_double_three_field: Bitfield::default(),
        }
    }

}

impl Patterns {

    pub const EMPTY_UNCHECKED_FIVE_POS: ColorContainer<Option<Pos>> = ColorContainer {
        black: None,
        white: None
    };

    #[inline]
    pub fn update_with_slice_mut<const C: Color, const D: Direction>(&mut self, slice: &mut Slice) {
        let slice_pattern = slice.calculate_slice_pattern::<C>();

        match (slice.pattern_bitmap.player::<C>() == 0, slice_pattern.is_empty()) {
            (false, true) => {
                self.clear_with_slice_mut::<C, D>(slice);
            },
            (_, false) => {
                self.update_with_slice_pattern_mut::<C, D>(slice, slice_pattern);
            },
            _ => {}
        };
    }

    #[inline]
    pub fn clear_with_slice_mut<const C: Color, const D: Direction>(&mut self, slice: &mut Slice) {
        let start_idx = slice.start_pos.idx_usize();
        let mut clear_mask = slice.pattern_bitmap.player::<C>();
        while clear_mask != 0 {
            let pattern_idx = clear_mask.trailing_zeros() as usize;
            clear_mask &= clear_mask - 1;

            let idx = step_idx!(D, start_idx, pattern_idx);
            self.field.player_ref_mut::<C>()[idx].apply_mask_mut::<C, D>(0);
        }

        self.score_table.clear_slice_mut::<C, D>(slice.idx as usize);

        *slice.pattern_bitmap.player_ref_mut::<C>() = 0;
    }

    #[inline]
    fn update_with_slice_pattern_mut<const C: Color, const D: Direction>(
        &mut self, slice: &mut Slice, slice_pattern: SlicePattern
    ) {
        *self.unchecked_five_pos.player_ref_mut::<C>() = self.unchecked_five_pos.player_ref::<C>().or(
            (slice_pattern.patterns & SLICE_PATTERN_FIVE_MASK != 0).then(|| {
                let slice_idx = (slice_pattern.patterns & SLICE_PATTERN_FIVE_MASK).trailing_zeros() / 8;
                Pos::from_index(step_idx!(D, slice.start_pos.idx(), slice_idx as u8))
            })
        );

        let mut three_mask = slice_pattern.patterns & SLICE_PATTERN_THREE_MASK;

        self.score_table.set_slice_mut::<C, D>(
            slice.idx as usize,
            three_mask.count_ones() as u8,
            (slice_pattern.patterns & SLICE_PATTERN_CLOSED_FOUR_MASK).count_ones() as u8,
            (slice_pattern.patterns & SLICE_PATTERN_OPEN_FOUR_MASK).count_ones() as u8
        );

        while C == Color::Black && three_mask != 0 {
            let three_slice_idx = three_mask.trailing_zeros() / 8;
            three_mask &= three_mask - 1;

            let three_idx: usize = step_idx!(D, slice.start_pos.idx_usize(), three_slice_idx as usize);
            if self.field.black[three_idx].has_three() {
                self.unchecked_double_three_field.set_mut(Pos::from_index(three_idx as u8));
            }
        }

        let pattern_bitmap = encode_u128_into_u16(slice_pattern.patterns);
        let slice_patterns = slice_pattern.patterns.to_ne_bytes();

        let start_idx = slice.start_pos.idx_usize();
        let mut update_mask = (slice.pattern_bitmap.player::<C>() ^ pattern_bitmap) | pattern_bitmap;
        while update_mask != 0 {
            let pattern_idx = update_mask.trailing_zeros() as usize;
            update_mask &= update_mask - 1;

            let idx = step_idx!(D, start_idx, pattern_idx);
            self.field.player_ref_mut::<C>()[idx].apply_mask_mut::<C, D>(slice_patterns[pattern_idx]);
        }

        self.unchecked_five_in_a_row = self.unchecked_five_in_a_row.or(
            contains_five_in_a_row(slice.stones::<C>())
                .then_some(C)
        );

        *slice.pattern_bitmap.player_ref_mut::<C>() = pattern_bitmap;
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

            Direction::from_pattern_position(tails)
        })
    }

}

fn encode_u128_into_u16(source: u128) -> u16 {
    Simd::<u8, 16>::from(source.to_ne_bytes())
        .simd_ne(Simd::splat(0))
        .to_bitmask() as u16 // _mm_movemask_epi8
}
