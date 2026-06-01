use crate::bitfield::Bitfield;
use crate::notation::color::{AlignedColorContainer, Color, ColorContainer};
use crate::notation::direction::{Direction, DirectionContainer};
use crate::notation::pos::{MaybePos, Pos};
use crate::notation::rule::{ForbiddenKind, RuleKind};
use crate::pattern_index::PatternIndex;
use crate::slice::Slice;
use crate::slice_pattern::SlicePattern;
use crate::utils::empty::Empty;
use crate::utils::lang::{repeat_16x, repeat_4x};
use crate::{assert_struct_sizes, slice_pattern, step_idx};

pub const CLOSED_FOUR_SINGLE: u8        = 0b1000_0000;
pub const CLOSED_FOUR_DOUBLE: u8        = 0b1100_0000;
pub const OPEN_FOUR: u8                 = 0b0010_0000;
pub const ANY_FOUR: u8                  = 0b1110_0000;
pub const OPEN_THREE: u8                = 0b0001_0000;

pub const CLOSE_THREE: u8               = 0b0000_1000;
pub const POTENTIAL_THREE :u8           = 0b0000_0100;
pub const POTENTIAL_FOUR :u8            = 0b0000_0010;

pub const FIVE: u8                      = 0b0000_0001;

pub const UNIT_CLOSED_FOUR_SINGLE_MASK: u32 = repeat_4x(CLOSED_FOUR_SINGLE);
pub const UNIT_CLOSED_FOUR_MASK: u32        = repeat_4x(CLOSED_FOUR_DOUBLE);
pub const UNIT_OPEN_FOUR_MASK: u32          = repeat_4x(OPEN_FOUR);
pub const UNIT_ANY_FOUR_MASK: u32           = repeat_4x(ANY_FOUR);

pub const UNIT_OPEN_THREE_MASK: u32         = repeat_4x(OPEN_THREE);
pub const UNIT_CLOSE_THREE_MASK: u32        = repeat_4x(CLOSE_THREE);

pub const UNIT_POTENTIAL_THREE_MASK: u32    = repeat_4x(POTENTIAL_THREE);
pub const UNIT_POTENTIAL_FOUR_MASK: u32     = repeat_4x(POTENTIAL_FOUR);
pub const UNIT_ANY_POTENTIAL_MASK: u32      = repeat_4x(POTENTIAL_FOUR | POTENTIAL_THREE);

pub const UNIT_FIVE_MASK: u32               = repeat_4x(FIVE);

pub const UNIT_TACTICAL_MASK: u32           = repeat_4x(OPEN_THREE | ANY_FOUR);

pub const SLICE_PATTERN_FIVE_MASK: u128     = repeat_16x(FIVE);

pub const PATTERN_SIZE: usize = 256;

#[derive(Debug, Copy, Clone, Default)]
pub struct Pattern(DirectionContainer<u8>);

assert_struct_sizes!(Pattern, size=4, align=1);

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

    pub fn has_open_three(&self) -> bool {
        self.apply_mask(UNIT_OPEN_THREE_MASK) != 0
    }

    pub fn has_open_threes(&self) -> bool {
        self.apply_mask(UNIT_OPEN_THREE_MASK).count_ones() > 1
    }

    pub fn has_any_four(&self) -> bool {
        self.apply_mask(UNIT_ANY_FOUR_MASK) != 0
    }

    pub fn has_open_four(&self) -> bool {
        self.apply_mask(UNIT_OPEN_FOUR_MASK) != 0
    }

    pub fn has_any_fours(&self) -> bool {
        self.apply_mask(UNIT_ANY_FOUR_MASK).count_ones() > 1
    }

    pub fn has_closed_four(&self) -> bool {
        self.apply_mask(UNIT_CLOSED_FOUR_MASK) != 0
    }

    pub fn has_close_three(&self) -> bool {
        self.apply_mask(UNIT_CLOSE_THREE_MASK) != 0
    }

    pub fn has_any_threat(&self) -> bool {
        self.apply_mask(UNIT_OPEN_THREE_MASK | UNIT_ANY_FOUR_MASK) != 0
    }

    pub fn has_five(&self) -> bool {
        self.apply_mask(UNIT_FIVE_MASK) != 0
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

    pub fn count_any_fours(&self) -> u32 {
        self.apply_mask(UNIT_ANY_FOUR_MASK).count_ones()
    }

    pub fn count_potential_three(&self) -> u32 {
        self.apply_mask(UNIT_POTENTIAL_THREE_MASK).count_ones()
    }

    pub fn count_potential_four(&self) -> u32 {
        self.apply_mask(UNIT_POTENTIAL_FOUR_MASK).count_ones()
    }
    
    pub fn count_five(&self) -> u32 {
        self.apply_mask(UNIT_FIVE_MASK).count_ones()
    }

    pub fn count_any_potential(&self) -> u32 {
        (
            self.apply_mask(UNIT_POTENTIAL_THREE_MASK)
                | (self.apply_mask(UNIT_POTENTIAL_FOUR_MASK) << 1)
        ).count_ones()
    }

    pub fn iter_three_directions(&self) -> impl Iterator<Item=Direction> + '_ {
        DirectionIterator { packed_unit: self.apply_mask(UNIT_OPEN_THREE_MASK) }
    }

    fn is_forbidden_unchecked(&self) -> bool {
        !self.has_five() && (self.has_any_fours() || self.has_open_threes())
    }

    fn apply_mask(&self, mask: u32) -> u32 {
        u32::from(*self) & mask
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Patterns {
    pub field: AlignedColorContainer<[Pattern; PATTERN_SIZE]>,
    pub indexes: ColorContainer<PatternIndex>,
    pub unchecked_five_pos: ColorContainer<MaybePos>,
    pub candidate_overline_field: Bitfield,
    pub candidate_forbidden_field: Bitfield,
    pub forbidden_field: Bitfield,
}

impl Empty for Patterns {
    fn empty() -> Self {
        Self {
            field: unsafe { std::mem::zeroed() },
            indexes: ColorContainer::new(PatternIndex::empty(), PatternIndex::empty()),
            unchecked_five_pos: ColorContainer::new(MaybePos::NONE, MaybePos::NONE),
            candidate_overline_field: Bitfield::ZERO_FILLED,
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

    pub fn forbidden_kind(&self, pos: Pos) -> Option<ForbiddenKind> {
        self.is_forbidden(pos).then(|| {
            if self.candidate_overline_field.is_hot(pos) {
                ForbiddenKind::Overline
            } else if self.field[Color::Black][pos.idx_usize()].has_any_fours() {
                ForbiddenKind::DoubleFour
            } else {
                ForbiddenKind::DoubleThree
            }
        })
    }

    pub fn effective_fork_four_field(&self, color: Color) -> Bitfield {
        let mut field = self.indexes[color].fork_fours;

        if color == Color::Black {
            field &= !self.forbidden_field;
        }

        field
    }

    pub fn effective_fork_three_four_field(&self, color: Color) -> Bitfield {
        let mut field = self.indexes[color].closed_fours & self.indexes[color].open_threes;

        if color == Color::Black {
            field &= !self.forbidden_field;
        }

        field
    }

    #[inline(always)]
    pub fn update_pattern_with_slice<const R: RuleKind, const C: Color, const D: Direction>(&mut self, slice: &mut Slice) -> u16 {
        let slice_pattern = slice.calculate_slice_pattern::<R, C>();

        let touched_bitmask = match (slice.pattern_bitmap[C] == 0, slice_pattern.is_empty()) {
            (false, true) => self.clear_pattern_with_slice::<R, C, D>(slice),
            (_, false) => self.update_with_slice_pattern::<R, C, D>(slice, slice_pattern),
            _ => 0
        };

        self.update_overline_field::<R, C, D>(slice);

        touched_bitmask
    }

    #[inline(always)]
    pub fn clear_pattern_with_slice<const R: RuleKind, const C: Color, const D: Direction>(&mut self, slice: &mut Slice) -> u16 {
        let start_idx = slice.start_pos.idx_usize();

        let old_bitmap = self.indexes[C]
            .replace_slice_bitmap::<D>(slice.idx, SlicePattern::EMPTY);
        let changed_bitmask = old_bitmap.changed_pattern_bitmap(SlicePattern::EMPTY);

        let mut clear_mask = changed_bitmask;
        while clear_mask != 0 {
            let slice_idx = clear_mask.trailing_zeros() as usize;
            clear_mask &= clear_mask - 1;

            let board_idx = step_idx!(D, start_idx, slice_idx);

            self.field[C][board_idx].0[D] = 0;
        }

        self.indexes[C]
            .update_slice_bitfields::<R, C, D>(&self.field[C], start_idx, old_bitmap, SlicePattern::EMPTY);

        changed_bitmask
    }

    #[inline(always)]
    fn update_with_slice_pattern<const R: RuleKind, const C: Color, const D: Direction>(
        &mut self, slice: &mut Slice, slice_pattern: SlicePattern
    ) -> u16 {
        if (slice_pattern.patterns & SLICE_PATTERN_FIVE_MASK) != 0 {
            let slice_idx = (slice_pattern.patterns & SLICE_PATTERN_FIVE_MASK).trailing_zeros() / 8;
            let pos = Pos::from_index(step_idx!(D, slice.start_pos.idx(), slice_idx as u8));

            self.unchecked_five_pos[C] = pos.into();
        }

        slice.pattern_bitmap[C] = slice_pattern.pattern_bitmap();
        let old_slice_bitmap = self.indexes[C]
            .replace_slice_bitmap::<D>(slice.idx, slice_pattern);

        let slice_patterns = slice_pattern.patterns.to_le_bytes();

        let changed_bitmask = old_slice_bitmap.changed_pattern_bitmap(slice_pattern);

        let start_idx = slice.start_pos.idx_usize();
        let mut update_bitmask = changed_bitmask;
        while update_bitmask != 0 {
            let slice_idx = update_bitmask.trailing_zeros() as usize;
            update_bitmask &= update_bitmask - 1;

            let board_idx = step_idx!(D, start_idx, slice_idx);

            self.field[C][board_idx].0[D] = slice_patterns[slice_idx];

            if C == Color::Black && R == RuleKind::Renju
                && self.field[Color::Black][board_idx].is_forbidden_unchecked() 
            {
                self.candidate_forbidden_field.set_idx(board_idx);
            }
        }

        self.indexes[C]
            .update_slice_bitfields::<R, C, D>(&self.field[C], start_idx, old_slice_bitmap, slice_pattern);

        changed_bitmask
    }

    #[inline(always)]
    fn update_overline_field<const R: RuleKind, const C: Color, const D: Direction>(&mut self, slice: &mut Slice) {
        if C == Color::Black && R == RuleKind::Renju {
            let mut overline_bitmask = slice_pattern::match_overline_positions(slice.stones[Color::Black], slice.blocks::<C>());
            while overline_bitmask != 0 {
                let slice_idx = overline_bitmask.trailing_zeros() as usize;
                overline_bitmask &= overline_bitmask - 1;

                let board_idx = step_idx!(D, slice.start_pos.idx_usize(), slice_idx);

                self.candidate_overline_field.set_idx(board_idx);
                self.candidate_forbidden_field.set_idx(board_idx);
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
