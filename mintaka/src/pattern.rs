use crate::cache::dummy_patch_cache::DummyPatchCache;
use crate::cache::patch_cache::PatchCache;
use crate::notation::color::Color;
use crate::notation::direction::Direction;
use crate::notation::pos;
use crate::notation::rule::ForbiddenKind;
use crate::slice::Slice;
use crate::slice_pattern::EMPTY_SLICE_PATCH;
use crate::{cartesian_to_index, pop_count_less_then_two, pop_count_less_then_two_unchecked};

pub const CLOSED_FOUR_SINGLE: u8    = 0b1000_0000;
pub const CLOSED_FOUR_DOUBLE: u8    = 0b1100_0000;
pub const OPEN_FOUR: u8             = 0b0010_0000;
pub const TOTAL_FOUR: u8            = 0b1110_0000;
pub const FIVE: u8                  = 0b0001_0000;

pub const OPEN_THREE: u8            = 0b0000_1000;
pub const CORE_THREE: u8            = 0b0000_0100;
pub const CLOSE_THREE: u8           = 0b0000_0010;
// invalid-3 for black, overline(black) for white
pub const INV_THREE_OVERLINE: u8    = 0b0000_0001;

const UNIT_CLOSED_FOUR_MASK: u32    = 0b1100_0000_1100_0000_1100_0000_1100_0000;
const UNIT_OPEN_FOUR_MASK: u32      = 0b0010_0000_0010_0000_0010_0000_0010_0000;
const UNIT_TOTAL_FOUR_MASK: u32     = 0b1110_0000_1110_0000_1110_0000_1110_0000;
const UNIT_FIVE_MASK: u32           = 0b0001_0000_0001_0000_0001_0000_0001_0000;

const UNIT_OPEN_THREE_MASK: u32     = 0b0000_1000_0000_1000_0000_1000_0000_1000;
const UNIT_CORE_THREE_MASK: u32     = 0b0000_0100_0000_0100_0000_0100_0000_0100;
const UNIT_CLOSE_THREE_MASK: u32    = 0b0000_0010_0000_0010_0000_0010_0000_0010;
const UNIT_INV_3_OVERLINE_MASK: u32 = 0b0000_0001_0000_0001_0000_0001_0000_0001;

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum PatternCount {
    Cold,
    Single,
    Multiple
}

impl PatternCount {

    fn from_masked_unit(packed: u32) -> Self {
        if packed == 0 {
            PatternCount::Cold
        } else if pop_count_less_then_two_unchecked!(packed) {
            PatternCount::Single
        } else {
            PatternCount::Multiple
        }
    }

}

// packed in 8-bit: closed-4-1 closed-4-2 open-4 five _ open-3 close-3 core-3 etc.
// total 32bit
#[derive(Debug, Copy, Clone, Default)]
pub struct PatternUnit {
    horizontal: u8,
    vertical: u8,
    ascending: u8,
    descending: u8
}

impl From<PatternUnit> for u32 {

    fn from(value: PatternUnit) -> Self {
        unsafe { std::mem::transmute(value) }
    }

}

impl PatternUnit {

    pub fn is_empty(&self) -> bool {
        u32::from(*self) == 0
    }

    pub fn has_threes(&self) -> bool {
        !pop_count_less_then_two!(self.apply_mask(UNIT_OPEN_THREE_MASK))
    }

    pub fn has_fours(&self) -> bool {
        !pop_count_less_then_two!(self.apply_mask(UNIT_TOTAL_FOUR_MASK))
    }

    pub fn has_five(&self) -> bool {
        self.apply_mask(UNIT_FIVE_MASK) != 0
    }

    pub fn count_threes(&self) -> PatternCount {
        PatternCount::from_masked_unit(self.apply_mask(UNIT_OPEN_THREE_MASK))
    }

    pub fn count_fours(&self) -> PatternCount {
        PatternCount::from_masked_unit(self.apply_mask(UNIT_TOTAL_FOUR_MASK))
    }

    pub fn open_three_at<const D: Direction>(&self) -> bool {
        self.with_mask_at::<D>(OPEN_THREE)
    }

    pub fn close_three_at<const D: Direction>(&self) -> bool {
        self.with_mask_at::<D>(CLOSE_THREE)
    }

    pub fn closed_four_at<const D: Direction>(&self) -> bool {
        self.with_mask_at::<D>(CLOSED_FOUR_SINGLE)
    }

    pub fn open_four_at<const D: Direction>(&self) -> bool {
        self.with_mask_at::<D>(OPEN_FOUR)
    }

    pub fn five_at<const D: Direction>(&self) -> bool {
        self.with_mask_at::<D>(FIVE)
    }

    fn with_mask_at<const D: Direction>(&self, mask: u8) -> bool {
        let packed = match D {
            Direction::Horizontal => self.horizontal,
            Direction::Vertical => self.vertical,
            Direction::Ascending => self.ascending,
            Direction::Descending => self.descending
        };

        packed & mask == mask
    }

    pub fn count_open_threes(&self) -> u32 {
        self.apply_mask(UNIT_OPEN_THREE_MASK).count_ones()
    }

    pub fn count_close_threes(&self) -> u32 {
        self.apply_mask(UNIT_CLOSE_THREE_MASK).count_ones()
    }

    pub fn count_core_threes(&self) -> u32 {
        self.apply_mask(UNIT_CORE_THREE_MASK).count_ones()
    }

    pub fn count_closed_fours(&self) -> u32 {
        self.apply_mask(UNIT_CLOSED_FOUR_MASK).count_ones()
    }

    pub fn count_open_fours(&self) -> u32 {
        self.apply_mask(UNIT_OPEN_FOUR_MASK).count_ones()
    }

    pub fn count_total_fours(&self) -> u32 {
        self.apply_mask(UNIT_TOTAL_FOUR_MASK).count_ones()
    }

    pub fn count_fives(&self) -> u32 {
        self.apply_mask(UNIT_FIVE_MASK).count_ones()
    }

    fn apply_mask(&self, mask: u32) -> u32 {
        u32::from(*self) & mask
    }

}

#[derive(Debug, Copy, Clone, Default)]
pub struct Pattern {
    pub black_unit: PatternUnit,
    pub white_unit: PatternUnit
}

impl From<Pattern> for u64 {

    fn from(value: Pattern) -> Self {
        unsafe { std::mem::transmute(value) }
    }

}

impl Pattern {

    pub fn access_unit(&self, color: Color) -> &PatternUnit {
        match color {
            Color::Black => &self.black_unit,
            Color::White => &self.white_unit
        }
    }

    pub fn is_empty(&self) -> bool {
        u64::from(*self) == 0
    }

    pub fn is_not_empty(&self) -> bool {
        u64::from(*self) != 0
    }

    pub fn is_forbidden(&self) -> bool {
        self.is_not_empty()
            && (self.black_unit.has_fours()
                || self.black_unit.has_threes()
                || self.has_overline())
            && !self.black_unit.has_five()
    }

    pub fn forbidden_kind(&self) -> Option<ForbiddenKind> {
        self.is_not_empty()
            .then(||
                if self.black_unit.has_threes() {
                    ForbiddenKind::DoubleThree
                } else if self.black_unit.has_fours() {
                    ForbiddenKind::DoubleFour
                } else {
                    ForbiddenKind::Overline
                }
            )
            .filter(|_| !self.black_unit.has_five())
    }

    #[inline(always)]
    pub fn apply_mask_mut<const C: Color, const D: Direction>(mut self, patch: u8) {
        match (C, D) {
            (Color::Black, Direction::Horizontal) => {
                self.black_unit.horizontal = patch
            },
            (Color::Black, Direction::Vertical) => {
                self.black_unit.vertical = patch
            },
            (Color::Black, Direction::Ascending) => {
                self.black_unit.ascending = patch
            },
            (Color::Black, Direction::Descending) => {
                self.black_unit.descending = patch
            },
            (Color::White, Direction::Horizontal) => {
                self.white_unit.horizontal = patch
            },
            (Color::White, Direction::Vertical) => {
                self.white_unit.vertical = patch
            },
            (Color::White, Direction::Ascending) => {
                self.white_unit.ascending = patch
            },
            (Color::White, Direction::Descending) => {
                self.white_unit.descending = patch
            },
        }
    }

    pub fn has_overline(&self) -> bool {
        false // TODO
    }

}

#[derive(Debug, Copy, Clone)]
pub struct Patterns {
    pub field: [Pattern; pos::BOARD_SIZE],
    pub five_in_a_row: Option<(Direction, u8, Color)>,
}

impl Default for Patterns {

    fn default() -> Self {
        Self {
            field: [Pattern::default(); pos::BOARD_SIZE],
            five_in_a_row: None,
        }
    }

}

impl Patterns {

    pub fn update_with_slice_mut<const D: Direction>(&mut self, slice: &Slice) {
        if slice.is_no_joy() {
            return
        }

        let mut patch_cache = DummyPatchCache {}; // TODO: DEBUG
        let slice_patch = patch_cache.probe_mut(slice.slice_key())
            .unwrap_or_else(|| {
                let patch = slice.calculate_slice_patch();
                patch_cache.put_mut(slice.slice_key(), patch);
                patch
            });

        if slice_patch == EMPTY_SLICE_PATCH {
            return
        }
        
        let black_patch = slice_patch.black_patch_as_u8_array();
        let white_patch = slice_patch.white_patch_as_u8_array();

        for offset in 0 .. slice.length {
            let idx = match D {
                Direction::Horizontal =>
                    cartesian_to_index!(slice.start_pos.row(), slice.start_pos.col() + offset),
                Direction::Vertical =>
                    cartesian_to_index!(slice.start_pos.row() + offset, slice.start_pos.col()),
                Direction::Ascending =>
                    cartesian_to_index!(slice.start_pos.row() + offset, slice.start_pos.col() + offset),
                Direction::Descending =>
                    cartesian_to_index!(slice.start_pos.row() - offset, slice.start_pos.col() + offset),
            } as usize;

            self.field[idx].apply_mask_mut::<{ Color::Black }, D>(black_patch[offset as usize]);
            self.field[idx].apply_mask_mut::<{ Color::White }, D>(white_patch[offset as usize]);
        }

        self.five_in_a_row = self.five_in_a_row.or_else(||
            slice_patch.five_in_a_row
                .map(|(idx, color)| (D, idx, color))
        );
    }

    pub fn validate_double_three_mut(&mut self) {
    }

}
