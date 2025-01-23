use crate::bitfield::Bitfield;
use crate::cartesian_to_index;
use crate::notation::color::Color;
use crate::notation::direction::Direction;
use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::notation::rule::ForbiddenKind;
use crate::slice::Slice;
use crate::slice_pattern::{contains_five_in_a_row, SlicePattern};
use crate::utils::lang_utils::repeat_4x;

pub const CLOSED_FOUR_SINGLE: u8        = 0b1000_0000;
pub const CLOSED_FOUR_DOUBLE: u8        = 0b1100_0000;
pub const OPEN_FOUR: u8                 = 0b0010_0000;
pub const TOTAL_FOUR: u8                = 0b1110_0000;
pub const FIVE: u8                      = 0b0001_0000;

pub const OPEN_THREE: u8                = 0b0000_1000;
pub const CLOSE_THREE: u8               = 0b0000_0100;
pub const OVERLINE: u8                  = 0b0000_0010;
pub const MARKER: u8                    = 0b0000_0001;

const OPEN_THREE_POSITION: u32          = 3;

const UNIT_CLOSED_FOUR_SINGLE_MASK: u32 = repeat_4x(CLOSED_FOUR_SINGLE);
const UNIT_CLOSED_FOUR_MASK: u32        = repeat_4x(CLOSED_FOUR_DOUBLE);
const UNIT_OPEN_FOUR_MASK: u32          = repeat_4x(OPEN_FOUR);
const UNIT_TOTAL_FOUR_MASK: u32         = repeat_4x(TOTAL_FOUR);
const UNIT_FIVE_MASK: u32               = repeat_4x(FIVE);

const UNIT_OPEN_THREE_MASK: u32         = repeat_4x(OPEN_THREE);
const UNIT_CLOSE_THREE_MASK: u32        = repeat_4x(CLOSE_THREE);
const UNIT_OVERLINE_MASK: u32           = repeat_4x(OVERLINE);

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum PatternCount {
    Cold,
    Single,
    Multiple
}

impl PatternCount {

    fn from_masked_unit(masked: u32) -> Self {
        if masked == 0 {
            PatternCount::Cold
        } else if masked.count_ones() < 2 {
            PatternCount::Single
        } else {
            PatternCount::Multiple
        }
    }

}

// packed in 8-bit: closed-4-1 closed-4-2 open-4 five open-3 close-3 overline open-3-direction
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

    pub fn has_three(&self) -> bool {
        self.apply_mask(UNIT_OPEN_THREE_MASK) != 0
    }

    pub fn has_threes(&self) -> bool {
        self.apply_mask(UNIT_OPEN_THREE_MASK).count_ones() > 1
    }

    pub fn has_any_four(&self) -> bool {
        self.apply_mask(UNIT_TOTAL_FOUR_MASK) != 0
    }

    pub fn has_open_four(&self) -> bool {
        self.apply_mask(UNIT_OPEN_FOUR_MASK) != 0
    }

    pub fn has_fours(&self) -> bool {
        self.apply_mask(UNIT_TOTAL_FOUR_MASK).count_ones() > 1
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

    pub fn iter_three_directions(&self) -> impl Iterator<Item=Direction> + '_ {
        ThreeDirectionIterator::from(self)
    }

    pub fn has_invalid_double_three(&self) -> bool {
        self.horizontal & MARKER == MARKER
    }

    pub fn mark_invalid_double_three(&mut self) {
        self.horizontal |= MARKER;
    }

    pub fn unmark_invalid_double_three(&mut self) {
        self.horizontal &= !MARKER;
    }
    
    pub fn closed_four_direction_unchecked(&self) -> Direction {
        let masked = self.apply_mask(UNIT_CLOSED_FOUR_SINGLE_MASK);
        
        const HORIZONTAL: u32 = CLOSED_FOUR_SINGLE as u32;
        const VERTICAL: u32 = (CLOSED_FOUR_SINGLE as u32) << 8;
        const ASCENDING: u32 = (CLOSED_FOUR_SINGLE as u32) << 16;
        const DESCENDING: u32 = (CLOSED_FOUR_SINGLE as u32) << 24;

        match masked { 
            HORIZONTAL => Direction::Horizontal,
            VERTICAL => Direction::Vertical,
            ASCENDING => Direction::Ascending,
            DESCENDING => Direction::Descending,
            _ => unreachable!()
        }
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

    pub fn access_unit(&self, color: Color) -> PatternUnit {
        match color {
            Color::Black => self.black_unit,
            Color::White => self.white_unit
        }
    }

    pub fn access_unit_pair(&self, color: Color) -> (PatternUnit, PatternUnit) {
        match color {
            Color::Black => (self.black_unit, self.white_unit),
            Color::White => (self.white_unit, self.black_unit)
        }
    }

    pub fn player_unit<const C: Color>(&self) -> PatternUnit {
        match C {
            Color::Black => self.black_unit,
            Color::White => self.white_unit
        }
    }

    pub fn opponent_unit<const C: Color>(&self) -> PatternUnit {
        match C {
            Color::Black => self.white_unit,
            Color::White => self.black_unit
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
            && (
                self.black_unit.has_fours()
                    || (self.black_unit.has_threes() && !self.black_unit.has_invalid_double_three())
                    || self.has_overline()
            )
            && !self.black_unit.has_five()
    }

    pub fn unit_by_color(&self, color: Color) -> PatternUnit {
        match color {
            Color::Black => self.black_unit,
            Color::White => self.white_unit,
        }
    }

    pub fn forbidden_kind(&self) -> Option<ForbiddenKind> {
        if self.is_forbidden() {
            if self.black_unit.has_threes() {
                Some(ForbiddenKind::DoubleThree)
            } else if self.black_unit.has_fours() {
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
        self.black_unit.has_overline()
    }

}

#[derive(Debug, Copy, Clone)]
pub struct Patterns {
    pub field: [Pattern; pos::BOARD_SIZE],
    pub five_in_a_row: Option<Color>,
    pub unchecked_double_three_field: Bitfield,
}

impl Default for Patterns {

    fn default() -> Self {
        Self {
            field: [Pattern::default(); pos::BOARD_SIZE],
            five_in_a_row: None,
            unchecked_double_three_field: Bitfield::default(),
        }
    }

}

impl Patterns {

    pub fn update_by_slice_mut<const C: Color, const D: Direction, const FULL_UPDATE: bool>(
        &mut self, slice: &Slice, slice_idx: usize
    ) {
        let slice_pattern = if slice.pattern_available::<C>() {
            slice.calculate_slice_pattern::<C, FULL_UPDATE>(slice_idx)
        } else {
            SlicePattern::EMPTY
        };

        for pattern_idx in
            if FULL_UPDATE { 0 .. slice.length as usize }
            else { slice_idx.saturating_sub(6) .. (slice_idx + 6).min(slice.length as usize) }
        {
            let idx = match D {
                Direction::Horizontal =>
                    cartesian_to_index!(slice.start_row as usize, slice.start_col as usize + pattern_idx),
                Direction::Vertical =>
                    cartesian_to_index!(slice.start_row as usize + pattern_idx, slice.start_col as usize),
                Direction::Ascending =>
                    cartesian_to_index!(slice.start_row as usize + pattern_idx, slice.start_col as usize + pattern_idx),
                Direction::Descending =>
                    cartesian_to_index!(slice.start_row as usize - pattern_idx, slice.start_col as usize + pattern_idx),
            };

            self.field[idx].apply_mask_mut::<C, D>(slice_pattern.patterns[pattern_idx]);

            if C == Color::Black {
                let pos = Pos::from_index(idx as u8);

                if self.field[idx].black_unit.has_threes() {
                    self.unchecked_double_three_field.set_mut(pos);
                } else {
                    self.unchecked_double_three_field.unset_mut(pos);
                }
            }
        }

        self.five_in_a_row = self.five_in_a_row.or(
            contains_five_in_a_row(slice.stones::<C>())
                .then_some(C)
        );
    }

}

struct ThreeDirectionIterator {
    packed_unit: u32
}

impl From<&PatternUnit> for ThreeDirectionIterator {

    fn from(value: &PatternUnit) -> Self {
        Self { packed_unit: value.apply_mask(UNIT_OPEN_THREE_MASK) }
    }

}

impl Iterator for ThreeDirectionIterator {

    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        (self.packed_unit != 0).then(|| {
            let tails = self.packed_unit.trailing_zeros();
            self.packed_unit &= self.packed_unit - 1;

            const HORIZONTAL_N: u32 = OPEN_THREE_POSITION;
            const VERTICAL_N: u32 = 8 + OPEN_THREE_POSITION;
            const ASCENDING_N: u32 = 8 * 2 + OPEN_THREE_POSITION;
            const DESCENDING_N: u32 = 8 * 3 + OPEN_THREE_POSITION;

            match tails {
                HORIZONTAL_N => Direction::Horizontal,
                VERTICAL_N => Direction::Vertical,
                ASCENDING_N => Direction::Ascending,
                DESCENDING_N => Direction::Descending,
                _ => unreachable!()
            }
        })
    }

}
