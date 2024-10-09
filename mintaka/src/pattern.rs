use crate::bitfield::{Bitfield, BitfieldOps};
use crate::cartesian_to_index;
use crate::memo::dummy_pattern_memo::DummySlicePatternMemo;
use crate::memo::slice_pattern_memo::SlicePatternMemo;
use crate::notation::color::Color;
use crate::notation::direction::Direction;
use crate::notation::pos;
use crate::notation::rule::ForbiddenKind;
use crate::slice::Slice;
use crate::utils::lang_utils::repeat_4x;
use ethnum::u256;

pub const CLOSED_FOUR_SINGLE: u8    = 0b1000_0000;
pub const CLOSED_FOUR_DOUBLE: u8    = 0b1100_0000;
pub const OPEN_FOUR: u8             = 0b0010_0000;
pub const TOTAL_FOUR: u8            = 0b1110_0000;
pub const FIVE: u8                  = 0b0001_0000;

pub const OPEN_THREE: u8            = 0b0000_1000;
pub const CLOSE_THREE: u8           = 0b0000_0100;
// invalid-3 for black, overline(black) for white
pub const INV_THREE_OVERLINE: u8    = 0b0000_0010;
pub const PADDING: u8               = 0b0000_0001;

const UNIT_CLOSED_FOUR_MASK: u32    = repeat_4x(CLOSED_FOUR_DOUBLE);
const UNIT_OPEN_FOUR_MASK: u32      = repeat_4x(OPEN_FOUR);
const UNIT_TOTAL_FOUR_MASK: u32     = repeat_4x(TOTAL_FOUR);
const UNIT_FIVE_MASK: u32           = repeat_4x(FIVE);

const UNIT_OPEN_THREE_MASK: u32     = repeat_4x(OPEN_THREE);
const UNIT_CLOSE_THREE_MASK: u32    = repeat_4x(CLOSE_THREE);
const UNIT_INV_3_OVERLINE_MASK: u32 = repeat_4x(INV_THREE_OVERLINE);

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
        } else if packed.count_ones() < 2 {
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
        unsafe { std::mem::transmute::<PatternUnit, u32>(value) }
    }

}

impl PatternUnit {

    pub fn is_empty(&self) -> bool {
        u32::from(*self) == 0
    }

    pub fn has_threes(&self) -> bool {
        self.apply_mask(UNIT_OPEN_THREE_MASK).count_ones() > 1
    }

    pub fn has_fours(&self) -> bool {
        self.apply_mask(UNIT_TOTAL_FOUR_MASK) > 1
    }

    pub fn has_close_three(&self) -> bool {
        self.apply_mask(UNIT_CLOSE_THREE_MASK) != 0
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
        unsafe { std::mem::transmute::<Pattern, u64>(value) }
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

    pub fn unit_by_color(&self, color: Color) -> PatternUnit {
        match color {
            Color::Black => self.black_unit,
            Color::White => self.white_unit,
        }
    }

    pub fn forbidden_kind(&self) -> Option<ForbiddenKind> {
        (self.is_not_empty() && self.is_forbidden())
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
    pub fn apply_mask_mut<const C: Color, const D: Direction>(&mut self, pattern: u8) {
        match (C, D) {
            (Color::Black, Direction::Horizontal) => self.black_unit.horizontal = pattern,
            (Color::Black, Direction::Vertical) => self.black_unit.vertical = pattern,
            (Color::Black, Direction::Ascending) => self.black_unit.ascending = pattern,
            (Color::Black, Direction::Descending) => self.black_unit.descending = pattern,
            (Color::White, Direction::Horizontal) => self.white_unit.horizontal = pattern,
            (Color::White, Direction::Vertical) => self.white_unit.vertical = pattern,
            (Color::White, Direction::Ascending) => self.white_unit.ascending = pattern,
            (Color::White, Direction::Descending) => self.white_unit.descending = pattern,
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
    double_three_field: Bitfield,
}

impl Default for Patterns {

    fn default() -> Self {
        Self {
            field: [Pattern::default(); pos::BOARD_SIZE],
            five_in_a_row: None,
            double_three_field: u256::MIN,
        }
    }

}

impl Patterns {

    pub fn update_by_slice_mut<const D: Direction>(&mut self, slice: &Slice) {
        if slice.is_no_joy() {
            return
        }

        let mut pattern_memo = DummySlicePatternMemo {}; // TODO: DEBUG
        let slice_pattern = pattern_memo.probe_or_put_mut(slice.raw_slice(), ||
            slice.calculate_slice_pattern()
        );

        if slice_pattern.is_empty() {
            return
        }
        
        for offset in 0 .. slice.length {
            let idx = match D {
                Direction::Horizontal =>
                    cartesian_to_index!(slice.start_row, slice.start_col + offset),
                Direction::Vertical =>
                    cartesian_to_index!(slice.start_row + offset, slice.start_col),
                Direction::Ascending =>
                    cartesian_to_index!(slice.start_row + offset, slice.start_col + offset),
                Direction::Descending =>
                    cartesian_to_index!(slice.start_row - offset, slice.start_col + offset),
            } as usize;

            self.field[idx].apply_mask_mut::<{ Color::Black }, D>(slice_pattern.black_patterns[offset as usize]);
            self.field[idx].apply_mask_mut::<{ Color::White }, D>(slice_pattern.white_patterns[offset as usize]);

            if self.field[idx].black_unit.has_threes() {
            }
        }

        self.five_in_a_row = self.five_in_a_row.or_else(||
            slice_pattern.five_in_a_row
                .map(|(idx, color)| (D, idx, color))
        );
    }

    pub fn validate_double_three_mut(&mut self) {
        for double_three_pos in self.double_three_field.iter_hot_pos() {
        }
    }

}
