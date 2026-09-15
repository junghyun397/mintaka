use crate::bitfield::Bitfield;
use crate::notation::color::{AlignedColorContainer, Color, ColorContainer};
use crate::notation::direction::{Direction, DirectionContainer};
use crate::notation::pos::{MaybePos, Pos};
use crate::notation::rule::{ForbiddenKind, RuleKind};
use crate::pattern_index::PatternIndex;
use crate::slice::Slice;
use crate::slice_pattern::SlicePattern;
use crate::utils::empty::Empty;
use crate::{assert_struct_sizes, repeat, slice_pattern, step_idx};

pub const CLOSED_FOURS: u8              = 0b1100_0000;
pub const CLOSED_FOUR_SINGLE: u8        = 0b1000_0000;

pub const OPEN_FOUR: u8                 = 0b0010_0000;
pub const ANY_FOUR: u8                  = 0b1110_0000;
pub const OPEN_THREE: u8                = 0b0001_0000;

pub const CLOSE_THREE: u8               = 0b0000_1000;
pub const POTENTIAL_THREE :u8           = 0b0000_0100;
pub const POTENTIAL_FOUR :u8            = 0b0000_0010;

pub const FIVE: u8                      = 0b0000_0001;

pub const PATTERN_SIZE: usize = 256;

#[derive(Debug, Copy, Clone, Default)]
#[repr(transparent)]
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

    pub fn has<const MASK: u8>(&self) -> bool {
        self.apply_mask(repeat!(MASK, x4 u32)) != 0
    }

    pub fn has_at<const MASK: u8>(&self, direction: Direction) -> bool {
        self.apply_mask((MASK as u32) << (direction as usize * 8)) != 0
    }

    pub fn count<const MASK: u8>(&self) -> u32 {
        self.apply_mask(repeat!(MASK, x4 u32)).count_ones()
    }

    pub fn has_open_three_at(&self, direction: Direction) -> bool {
        self.has_at::<OPEN_THREE>(direction)
    }

    pub fn is_tactical(&self) -> bool {
        self.has::<{ OPEN_THREE | ANY_FOUR }>()
    }

    pub fn has_closed_four(&self) -> bool {
        self.has::<CLOSED_FOUR_SINGLE>()
    }

    pub fn has_open_three(&self) -> bool {
        self.has::<OPEN_THREE>()
    }

    pub fn has_close_three(&self) -> bool {
        self.has::<CLOSE_THREE>()
    }

    pub fn has_five(&self) -> bool {
        self.has::<FIVE>()
    }

    pub fn has_any_four(&self) -> bool {
        self.has::<ANY_FOUR>()
    }

    pub fn has_open_four(&self) -> bool {
        self.has::<OPEN_FOUR>()
    }

    pub fn has_open_threes(&self) -> bool {
        self.count_open_three() > 1
    }

    pub fn has_any_fours(&self) -> bool {
        self.count_any_four() > 1
    }

    pub fn count_open_three(&self) -> u32 {
        self.count::<OPEN_THREE>()
    }

    pub fn count_close_three(&self) -> u32 {
        self.count::<CLOSE_THREE>()
    }

    pub fn count_closed_four(&self) -> u32 {
        self.count::<CLOSED_FOURS>()
    }

    pub fn count_open_four(&self) -> u32 {
        self.count::<OPEN_FOUR>()
    }

    pub fn count_any_four(&self) -> u32 {
        self.count::<ANY_FOUR>()
    }

    pub fn count_potential_three(&self) -> u32 {
        self.count::<POTENTIAL_THREE>()
    }

    pub fn count_potential_four(&self) -> u32 {
        self.count::<POTENTIAL_FOUR>()
    }

    pub fn count_five(&self) -> u32 {
        self.count::<FIVE>()
    }

    pub fn iter_three_directions(&self) -> impl Iterator<Item=Direction> + '_ {
        DirectionIterator { packed_unit: self.apply_mask(repeat!(OPEN_THREE, x4 u32)) }
    }

    pub fn iter_potential_four_directions(&self) -> impl Iterator<Item=Direction> + '_ {
        DirectionIterator { packed_unit: self.apply_mask(repeat!(POTENTIAL_FOUR, x4 u32)) }
    }

    fn is_forbidden_unchecked(&self) -> bool {
        !self.has_five() && (self.has_any_fours() || self.has_open_threes())
    }

    fn apply_mask(&self, mask: u32) -> u32 {
        u32::from(*self) & mask
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Patterns<const R: RuleKind> {
    pub field: AlignedColorContainer<[Pattern; PATTERN_SIZE]>,
    pub indexes: ColorContainer<PatternIndex<R>>,
    pub five_pos: ColorContainer<MaybePos>,
    pub candidate_overline_field: Bitfield,
    pub candidate_forbidden_field: Bitfield,
    pub forbidden_field: Bitfield,
}

impl<const R: RuleKind> Empty for Patterns<R> {
    fn empty() -> Self {
        Self {
            field: unsafe { std::mem::zeroed() },
            indexes: ColorContainer::new(PatternIndex::empty(), PatternIndex::empty()),
            five_pos: ColorContainer::new(MaybePos::NONE, MaybePos::NONE),
            candidate_overline_field: Bitfield::ZERO_FILLED,
            candidate_forbidden_field: Bitfield::ZERO_FILLED,
            forbidden_field: Bitfield::ZERO_FILLED,
        }
    }
}

impl<const R: RuleKind> Patterns<R> {
    #[inline(always)]
    pub fn is_forbidden(&self, pos: Pos) -> bool {
        R == RuleKind::Renju && self.forbidden_field.is_hot(pos)
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

    #[inline(never)] // reduce I-cache pressure
    pub fn update_pattern_with_slice<const C: Color, const D: Direction>(&mut self, slice: &mut Slice) -> u16 {
        let slice_pattern = slice.calculate_slice_pattern::<R, C>();

        let touched_bitmask = match (slice.pattern_bitmap[C] == 0, slice_pattern.is_empty()) {
            (false, true) => self.clear_pattern_with_slice::<C, D>(slice),
            (_, false) => self.update_with_slice_pattern::<C, D>(slice, slice_pattern),
            _ => 0
        };

        self.update_overline_field::<C, D>(slice);

        touched_bitmask
    }

    #[inline(never)] // reduce I-cache pressure
    pub fn clear_pattern_with_slice<const C: Color, const D: Direction>(&mut self, slice: &mut Slice) -> u16 {
        let start_idx = slice.start_pos.idx_usize();

        slice.pattern_bitmap[C] = 0;
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
            .update_slice_bitfields::<C, D>(&self.field[C], start_idx, old_bitmap, SlicePattern::EMPTY);

        changed_bitmask
    }

    #[inline(always)]
    fn update_with_slice_pattern<const C: Color, const D: Direction>(
        &mut self, slice: &mut Slice, slice_pattern: SlicePattern
    ) -> u16 {
        const SLICE_PATTERN_FIVE_MASK: u128 = repeat!(FIVE, x16 u128);

        if (slice_pattern.patterns & SLICE_PATTERN_FIVE_MASK) != 0 {
            let slice_idx = (slice_pattern.patterns & SLICE_PATTERN_FIVE_MASK).trailing_zeros() / 8;
            let pos = Pos::from_index(step_idx!(D, slice.start_pos.idx(), slice_idx as u8));

            self.five_pos[C] = pos.into();
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
            .update_slice_bitfields::<C, D>(&self.field[C], start_idx, old_slice_bitmap, slice_pattern);

        changed_bitmask
    }

    #[inline(always)]
    fn update_overline_field<const C: Color, const D: Direction>(&mut self, slice: &mut Slice) {
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
