use crate::bitfield::Bitfield;
use crate::notation::color::{Color, ColorContainer};
use crate::notation::direction::Direction;
use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::notation::rule::ForbiddenKind;
use crate::slice::Slice;
use crate::slice_pattern::{contains_five_in_a_row, SlicePattern};
use crate::utils::lang_utils::{repeat_16x, repeat_4x};

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
const CLOSED_FOUR_SINGLE_POSITION: u32  = 8;

const UNIT_CLOSED_FOUR_SINGLE_MASK: u32 = repeat_4x(CLOSED_FOUR_SINGLE);
const UNIT_CLOSED_FOUR_MASK: u32        = repeat_4x(CLOSED_FOUR_DOUBLE);
const UNIT_OPEN_FOUR_MASK: u32          = repeat_4x(OPEN_FOUR);
const UNIT_TOTAL_FOUR_MASK: u32         = repeat_4x(TOTAL_FOUR);
const UNIT_FIVE_MASK: u32               = repeat_4x(FIVE);

const UNIT_OPEN_THREE_MASK: u32         = repeat_4x(OPEN_THREE);
const UNIT_CLOSE_THREE_MASK: u32        = repeat_4x(CLOSE_THREE);
const UNIT_OVERLINE_MASK: u32           = repeat_4x(OVERLINE);

pub const SLICE_PATTERN_FIVE_MASK: u128 = repeat_16x(FIVE);

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

    pub fn has_three_four_fork(&self) -> bool {
        self.apply_mask(UNIT_TOTAL_FOUR_MASK) != 0 && self.apply_mask(UNIT_OPEN_THREE_MASK) != 0
    }

    pub fn count_threes(&self) -> PatternCount {
        PatternCount::from_masked_unit(self.apply_mask(UNIT_OPEN_THREE_MASK))
    }

    pub fn count_fours(&self) -> PatternCount {
        PatternCount::from_masked_unit(self.apply_mask(UNIT_TOTAL_FOUR_MASK))
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
        DirectionIterator::<OPEN_THREE_POSITION>::from(self)
    }

    pub fn iter_four_directions(&self) -> impl Iterator<Item=Direction> + '_ {
        DirectionIterator::<CLOSED_FOUR_SINGLE_POSITION>::from(self)
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

pub type Pattern = ColorContainer<PatternUnit>;

impl From<Pattern> for u64 {

    fn from(value: Pattern) -> Self {
        unsafe { std::mem::transmute::<Pattern, u64>(value) }
    }

}

impl Pattern {

    pub fn is_empty(&self) -> bool {
        u64::from(*self) == 0
    }

    pub fn is_not_empty(&self) -> bool {
        u64::from(*self) != 0
    }

    pub fn is_forbidden(&self) -> bool {
        self.is_not_empty()
            && (
                self.black.has_fours()
                    || (self.black.has_threes() && !self.black.has_invalid_double_three())
                    || self.has_overline()
            )
            && !self.black.has_five()
    }

    pub fn unit_by_color(&self, color: Color) -> PatternUnit {
        match color {
            Color::Black => self.black,
            Color::White => self.white,
        }
    }

    pub fn forbidden_kind(&self) -> Option<ForbiddenKind> {
        if self.is_forbidden() {
            if self.black.has_threes() {
                Some(ForbiddenKind::DoubleThree)
            } else if self.black.has_fours() {
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
            (Color::Black, Direction::Horizontal) => self.black.horizontal = pattern,
            (Color::Black, Direction::Vertical) => self.black.vertical = pattern,
            (Color::Black, Direction::Ascending) => self.black.ascending = pattern,
            (Color::Black, Direction::Descending) => self.black.descending = pattern,
            (Color::White, Direction::Horizontal) => self.white.horizontal = pattern,
            (Color::White, Direction::Vertical) => self.white.vertical = pattern,
            (Color::White, Direction::Ascending) => self.white.ascending = pattern,
            (Color::White, Direction::Descending) => self.white.descending = pattern,
        }
    }

    pub fn has_overline(&self) -> bool {
        self.black.has_overline()
    }

}

#[derive(Debug, Copy, Clone)]
pub struct Patterns {
    pub field: [Pattern; pos::BOARD_SIZE],
    pub five_in_a_row: Option<Color>,
    pub unchecked_double_three_field: Bitfield,
    pub unchecked_five_pos: ColorContainer<Option<Pos>>,
}

impl Default for Patterns {

    fn default() -> Self {
        Self {
            field: [Pattern::default(); pos::BOARD_SIZE],
            five_in_a_row: None,
            unchecked_double_three_field: Bitfield::default(),
            unchecked_five_pos: ColorContainer {
                black: None,
                white: None
            }
        }
    }

}

impl Patterns {

    pub const EMPTY_UNCHECKED_FIVE_POS: ColorContainer<Option<Pos>> = ColorContainer {
        black: None,
        white: None
    };

    pub fn update_by_slice_mut<const C: Color, const D: Direction, const FULL_UPDATE: bool>(
        &mut self, slice: &Slice, slice_idx: usize
    ) {
        if slice.pattern_available::<C>() {
            self.update_by_slice_pattern_mut::<C, D, FULL_UPDATE>(slice, slice_idx, slice.calculate_slice_pattern::<C, FULL_UPDATE>(slice_idx));
        } else {
            self.clear_by_slice_mut::<C, D, FULL_UPDATE>(slice, slice_idx);
        };
    }

    #[inline(always)]
    fn clear_by_slice_mut<const C: Color, const D: Direction, const FULL_UPDATE: bool>(&mut self, slice: &Slice, slice_idx: usize) {
        for pattern_idx in
            if FULL_UPDATE { 0 .. slice.length as usize }
            else { slice_idx.saturating_sub(6) .. (slice_idx + 6).min(slice.length as usize) }
        {
            self.field[slice.calculate_slice_offset::<D>(pattern_idx)].apply_mask_mut::<C, D>(0);
        }

        self.five_in_a_row = None;
    }

    #[inline(always)]
    fn update_by_slice_pattern_mut<const C: Color, const D: Direction, const FULL_UPDATE: bool>(
        &mut self, slice: &Slice, slice_idx: usize, slice_pattern: SlicePattern
    ) {
        *self.unchecked_five_pos.player_unit_mut::<C>() = self.unchecked_five_pos.player_unit::<C>().or(
            (slice_pattern.patterns & SLICE_PATTERN_FIVE_MASK != 0)
                .then(|| Pos::from_index(
                    slice.calculate_slice_offset::<D>(((slice_pattern.patterns & SLICE_PATTERN_FIVE_MASK).trailing_zeros() / 8) as usize) as u8
                ))
        );

        let slice_pattern = unsafe { std::mem::transmute::<SlicePattern, [u8; 16]>(slice_pattern) };

        for pattern_idx in
            if FULL_UPDATE { 0 .. slice.length as usize }
            else { slice_idx.saturating_sub(6) .. (slice_idx + 6).min(slice.length as usize) }
        {
            let idx = slice.calculate_slice_offset::<D>(pattern_idx);

            self.field[idx].apply_mask_mut::<C, D>(unsafe { std::ptr::read(slice_pattern.as_ptr().add(pattern_idx)) });

            if C == Color::Black {
                let pos = Pos::from_index(idx as u8);

                if self.field[idx].black.has_threes() {
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

struct DirectionIterator<const P: u32> {
    packed_unit: u32
}

impl<const P: u32> DirectionIterator<P> {

    const HORIZONTAL_N: u32 = P;
    const VERTICAL_N: u32 = 8 + P;
    const ASCENDING_N: u32 = 8 * 2 + P;
    const DESCENDING_N: u32 = 8 * 3 + P;

}

impl<const P: u32> From<&PatternUnit> for DirectionIterator<P> {

    fn from(value: &PatternUnit) -> Self {
        Self { packed_unit: value.apply_mask(UNIT_OPEN_THREE_MASK) }
    }

}

impl<const P: u32> Iterator for DirectionIterator<P> {

    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        (self.packed_unit != 0).then(|| {
            let tails = self.packed_unit.trailing_zeros();
            self.packed_unit &= self.packed_unit - 1;

            match tails {
                c if c == Self::HORIZONTAL_N => Direction::Horizontal,
                c if c == Self::VERTICAL_N => Direction::Vertical,
                c if c == Self::ASCENDING_N => Direction::Ascending,
                c if c == Self::DESCENDING_N => Direction::Descending,
                _ => unreachable!()
            }
        })
    }

}
