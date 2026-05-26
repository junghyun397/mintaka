use crate::bitfield::Bitfield;
use crate::notation::color::{AlignedColorContainer, Color, ColorContainer};
use crate::notation::direction::Direction;
use crate::notation::pos::{MaybePos, Pos};
use crate::notation::rule::{ForbiddenKind, RuleKind};
use crate::pattern_index::{pattern_bitmaps_from_patterns, PatternIndex, SliceBitmap};
use crate::slice::Slice;
use crate::slice_pattern::SlicePattern;
use crate::step_idx;
use crate::utils::empty::Empty;
use crate::utils::lang::{repeat_16x, repeat_4x};

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
#[repr(C, packed)]
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

impl From<u32> for Pattern {
    fn from(value: u32) -> Self {
        unsafe { std::mem::transmute::<u32, Pattern>(value) }
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

    pub fn has_closed_four(&self) -> bool {
        self.apply_mask(UNIT_CLOSED_FOUR_MASK) != 0
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
    pub indexes: ColorContainer<PatternIndex>,
    pub unchecked_five_pos: ColorContainer<MaybePos>,
    pub candidate_forbidden_field: Bitfield,
    pub forbidden_field: Bitfield,
}

impl Empty for Patterns {
    fn empty() -> Self {
        Self {
            field: unsafe { std::mem::zeroed() },
            indexes: ColorContainer::new(PatternIndex::empty(), PatternIndex::empty()),
            unchecked_five_pos: ColorContainer::new(MaybePos::NONE, MaybePos::NONE),
            candidate_forbidden_field: Bitfield::ZERO_FILLED,
            forbidden_field: Bitfield::ZERO_FILLED,
        }
    }
}

impl Patterns {
    #[inline(always)]
    pub fn is_forbidden(&self, pos: Pos) -> bool {
        self.forbidden_field.is_hot(pos)
    }

    #[inline(always)]
    pub fn forbidden_kind(&self, pos: Pos) -> Option<ForbiddenKind> {
        self.is_forbidden(pos).then(|| {
            let pattern = self.field[Color::Black][pos.idx_usize()];

            if pattern.has_overline() {
                ForbiddenKind::Overline
            } else if pattern.has_fours() {
                ForbiddenKind::DoubleFour
            } else {
                ForbiddenKind::DoubleThree
            }
        })
    }

    #[inline(always)]
    pub fn update_with_slice<const R: RuleKind, const C: Color, const D: Direction>(&mut self, slice: &mut Slice) {
        let slice_pattern = slice.calculate_slice_pattern::<R, C>();

        match (slice.pattern_bitmap[C] == 0, slice_pattern.is_empty()) {
            (false, true) =>
                self.clear_with_slice::<C, D>(slice),
            (_, false) =>
                self.update_with_slice_pattern::<C, D>(slice, slice_pattern),
            _ => {}
        };
    }

    #[inline(always)]
    pub fn clear_with_slice<const C: Color, const D: Direction>(&mut self, slice: &mut Slice) {
        let start_idx = slice.start_pos.idx_usize();

        let mut clear_mask = slice.pattern_bitmap[C];
        while clear_mask != 0 {
            let slice_idx = clear_mask.trailing_zeros() as usize;
            clear_mask &= clear_mask - 1;

            self.field[C][step_idx!(D, start_idx, slice_idx)]
                .apply_mask_mut::<D>(0);
        }

        let old_bitmap = self.indexes[C]
            .replace_slice_bitmap::<D>(slice.idx, SliceBitmap::empty());
        self.indexes[C]
            .update_slice_bitfields::<C, D>(&self.field[C], start_idx, old_bitmap, SliceBitmap::empty());
    }

    #[inline(always)]
    fn update_with_slice_pattern<const C: Color, const D: Direction>(
        &mut self, slice: &mut Slice, slice_pattern: SlicePattern
    ) {
        if (slice_pattern.patterns & SLICE_PATTERN_FIVE_MASK) != 0 {
            let slice_idx = (slice_pattern.patterns & SLICE_PATTERN_FIVE_MASK).trailing_zeros() / 8;
            let pos = Pos::from_index(step_idx!(D, slice.start_pos.idx(), slice_idx as u8));

            self.unchecked_five_pos[C] = pos.into();
        }

        let slice_patterns = slice_pattern.patterns.to_le_bytes();
        let (new_pattern_bitmap, new_slice_bitmap) = pattern_bitmaps_from_patterns(slice_patterns);

        let old_pattern_bitmap = std::mem::replace(&mut slice.pattern_bitmap[C], new_pattern_bitmap);

        let old_slice_bitmap = self.indexes[C]
            .replace_slice_bitmap::<D>(slice.idx, new_slice_bitmap);

        let start_idx = slice.start_pos.idx_usize();

        let mut update_bitmask = new_pattern_bitmap | old_pattern_bitmap;
        while update_bitmask != 0 {
            let slice_idx = update_bitmask.trailing_zeros() as usize;
            update_bitmask &= update_bitmask - 1;

            let board_idx = step_idx!(D, start_idx, slice_idx);

            self.field[C][board_idx]
                .apply_mask_mut::<D>(slice_patterns[slice_idx]);

            if C == Color::Black && self.field[Color::Black][board_idx].is_forbidden_unchecked() {
                self.candidate_forbidden_field.set_idx(board_idx);
            }
        }

        self.indexes[C]
            .update_slice_bitfields::<C, D>(&self.field[C], start_idx, old_slice_bitmap, new_slice_bitmap);
    }

    pub fn effective_fork_fours(&self, color: Color) -> u32 {
        match color {
            Color::Black => (self.indexes[Color::Black].fork_fours & !self.forbidden_field).count_hots(),
            Color::White => self.indexes[Color::White].fork_fours.count_hots()
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
