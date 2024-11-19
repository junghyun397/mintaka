use crate::bitfield::{Bitfield, BitfieldOps};
use crate::memo::dummy_pattern_memo::DummySlicePatternMemo;
use crate::memo::hash_key::HashKey;
use crate::memo::slice_pattern_memo::SlicePatternMemo;
use crate::notation::color::Color;
use crate::notation::direction::Direction;
use crate::notation::pos::{Pos, INVALID_POS};
use crate::pattern::Patterns;
use crate::slice::{Slice, Slices};
use ethnum::U256;

// 2256-bytes
#[derive(Copy, Clone)]
pub struct Board {
    pub player_color: Color,
    pub slices: Slices,
    pub patterns: Patterns,
    pub hot_field: Bitfield,
    pub stones: u8,
    pub hash_key: HashKey,
}

impl Default for Board {

    fn default() -> Self {
        Self {
            player_color: Color::Black,
            slices: Slices::default(),
            patterns: Patterns::default(),
            hot_field: U256::MIN,
            stones: 0,
            hash_key: HashKey::default()
        }
    }

}

impl Board {

    pub fn opponent_color(&self) -> Color {
        self.player_color.reversed()
    }

    pub fn stone_kind(&self, pos: Pos) -> Option<Color> {
        self.slices.vertical_slices[pos.row_usize()].stone_kind(pos.col())
    }

    pub fn set(mut self, pos: Pos) -> Self {
        self.set_mut(&mut DummySlicePatternMemo, pos);
        self
    }

    pub fn unset(mut self, pos: Pos) -> Self {
        self.unset_mut(&mut DummySlicePatternMemo, pos);
        self
    }

    pub fn pass(mut self) -> Self {
        self.pass_mut();
        self
    }

    pub fn set_mut(&mut self, memo: &mut impl SlicePatternMemo, pos: Pos) {
        self.incremental_update_mut(memo, pos, Slice::set_mut);

        self.stones += 1;
        self.hot_field.set(pos);
        self.switch_player_mut();
        self.hash_key = self.hash_key.set(self.player_color, pos);
    }

    pub fn unset_mut(&mut self, memo: &mut impl SlicePatternMemo, pos: Pos) {
        self.patterns.five_in_a_row = None;

        self.incremental_update_mut(memo, pos, Slice::unset_mut);

        self.stones -= 1;
        self.hot_field.unset(pos);
        self.switch_player_mut();
        self.hash_key = self.hash_key.set(self.player_color, pos);
    }

    pub fn pass_mut(&mut self) {
        self.switch_player_mut();
    }

    pub fn batch_set_mut(&mut self, moves: Box<[Pos]>) {
        let (black_moves, white_moves): (Vec<Pos>, Vec<Pos>) = moves.iter()
            .enumerate()
            .fold(
                (Vec::with_capacity(moves.len() / 2), Vec::with_capacity(moves.len() / 2)),
                |(mut even, mut odd), (idx, pos)| {
                    if idx % 2 == 0 {
                        even.push(*pos);
                    } else {
                        odd.push(*pos);
                    }

                    (even, odd)
                }
            );

        let player = Color::player_color_from_each_moves(black_moves.len(), white_moves.len());

        self.batch_set_each_color_mut(black_moves.into_boxed_slice(), white_moves.into_boxed_slice(), player)
    }

    pub fn batch_set_each_color_mut(&mut self, blacks: Box<[Pos]>, whites: Box<[Pos]>, player: Color) {
        self.stones += blacks.len() as u8 + whites.len() as u8;

        for pos in blacks {
            self.slices.set_mut(Color::Black, pos);
            self.hot_field.set(pos);
        }

        for pos in whites {
            self.slices.set_mut(Color::White, pos);
            self.hot_field.set(pos);
        }

        self.player_color = player;

        self.full_update_mut();
        self.hash_key = HashKey::from(&self.slices.horizontal_slices);
    }

    fn switch_player_mut(&mut self) {
        self.player_color = self.opponent_color();
    }

    #[cfg(not(feature = "prefetch_slice"))]
    fn incremental_update_mut(&mut self, memo: &mut impl SlicePatternMemo, pos: Pos, slice_mut_op: fn(&mut Slice, Color, u8)) {
        let horizontal_slice = &mut self.slices.horizontal_slices[pos.row_usize()];
        slice_mut_op(horizontal_slice, self.player_color, pos.col());
        self.patterns.update_by_slice_mut::<{ Direction::Horizontal }>(memo, horizontal_slice);

        let vertical_slice = &mut self.slices.vertical_slices[pos.col_usize()];
        slice_mut_op(vertical_slice, self.player_color, pos.row());
        self.patterns.update_by_slice_mut::<{ Direction::Vertical }>(memo, vertical_slice);

        if let Some(ascending_slice_idx) = Slices::ascending_slice_idx(pos) {
            let ascending_slice = &mut self.slices.ascending_slices[ascending_slice_idx];
            slice_mut_op(ascending_slice, self.player_color, pos.col() - ascending_slice.start_col);
            self.patterns.update_by_slice_mut::<{ Direction::Ascending }>(memo, ascending_slice);
        }

        if let Some(descending_slice_idx) = Slices::descending_slice_idx(pos) {
            let descending_slice = &mut self.slices.descending_slices[descending_slice_idx];
            slice_mut_op(descending_slice, self.player_color, pos.col() - descending_slice.start_col);
            self.patterns.update_by_slice_mut::<{ Direction::Descending }>(memo, descending_slice);
        }

        self.validate_double_three_mut();
    }

    #[cfg(feature = "prefetch_slice")]
    fn incremental_update_mut(&mut self, memo: &mut impl SlicePatternMemo, pos: Pos, slice_mut_op: fn(&mut Slice, Color, u8)) {
        let horizontal_slice = &mut self.slices.horizontal_slices[pos.row_usize()];
        slice_mut_op(horizontal_slice, self.player_color, pos.col());
        let is_horizontal_slice_valid = horizontal_slice.is_valid_pattern();

        let vertical_slice = &mut self.slices.vertical_slices[pos.col_usize()];
        slice_mut_op(vertical_slice, self.player_color, pos.row());
        let is_vertical_slice_valid = vertical_slice.is_valid_pattern();

        let valid_ascending_slice =
            if let Some(idx) = Slices::ascending_slice_idx(pos) {
                let ascending_slice = &mut self.slices.ascending_slices[idx];
                slice_mut_op(ascending_slice, self.player_color, pos.col() - ascending_slice.start_col);
                ascending_slice.is_valid_pattern().then_some(ascending_slice)
            } else {
                None
            };

        let valid_descending_slice =
            if let Some(idx) = Slices::descending_slice_idx(pos) {
                let descending_slice = &mut self.slices.descending_slices[idx];
                slice_mut_op(descending_slice, self.player_color, pos.col() - descending_slice.start_col);
                descending_slice.is_valid_pattern().then_some(descending_slice)
            } else {
                None
            };

        let mut prefetch_stack: [u64; 3] = [0, 0, 0];
        let mut prefetch_top: usize = 0;

        if is_vertical_slice_valid {
            prefetch_stack[prefetch_top] = vertical_slice.packed_slice();
            prefetch_top += 1;
        }

        if valid_ascending_slice.as_ref()
            .map_or(false, |slice| slice.is_valid_pattern())
        {
            prefetch_stack[prefetch_top] = valid_ascending_slice.as_ref().unwrap().packed_slice();
            prefetch_top += 1;
        }

        if valid_descending_slice.as_ref()
            .map_or(false, |slice| slice.is_valid_pattern())
        {
            prefetch_stack[prefetch_top] = valid_descending_slice.as_ref().unwrap().packed_slice();
            prefetch_top += 1;
        }

        if is_horizontal_slice_valid {
            if prefetch_top != 0 {
                prefetch_top -= 1;
                memo.prefetch_memo(prefetch_stack[prefetch_top]);
            }

            self.patterns.update_by_slice_mut::<{ Direction::Horizontal }>(memo, horizontal_slice);
        }

        if is_vertical_slice_valid {
            if prefetch_top != 0 {
                prefetch_top -= 1;
                memo.prefetch_memo(prefetch_stack[prefetch_top]);
            }

            self.patterns.update_by_slice_mut::<{ Direction::Vertical }>(memo, vertical_slice);
        }

        if let Some(ascending_slice) = valid_ascending_slice {
            if prefetch_top != 0 {
                memo.prefetch_memo(prefetch_stack[prefetch_top]);
            }

            self.patterns.update_by_slice_mut::<{ Direction::Ascending }>(memo, ascending_slice);
        }

        if let Some(descending_slice) = valid_descending_slice {
            self.patterns.update_by_slice_mut::<{ Direction::Descending }>(memo, descending_slice);
        }

        self.validate_double_three_mut();
    }

    fn full_update_mut(&mut self) {
        for horizontal_slice in self.slices.horizontal_slices.iter() {
            if horizontal_slice.is_valid_pattern() {
                self.patterns.update_by_slice_mut::<{ Direction::Horizontal }>(&mut DummySlicePatternMemo, horizontal_slice);
            }
        }

        for vertical_slice in self.slices.vertical_slices.iter() {
            if vertical_slice.is_valid_pattern() {
                self.patterns.update_by_slice_mut::<{ Direction::Vertical }>(&mut DummySlicePatternMemo, vertical_slice);
            }
        }

        for ascending_slice in self.slices.ascending_slices.iter() {
            if ascending_slice.is_valid_pattern() {
                self.patterns.update_by_slice_mut::<{ Direction::Ascending }>(&mut DummySlicePatternMemo, ascending_slice);
            }
        }

        for descending_slice in self.slices.descending_slices.iter() {
            if descending_slice.is_valid_pattern() {
                self.patterns.update_by_slice_mut::<{ Direction::Descending }>(&mut DummySlicePatternMemo, descending_slice);
            }
        }

        self.validate_double_three_mut();
    }

    fn validate_double_three_mut(&mut self) {
        for double_three_pos in self.patterns.unchecked_double_three_field.iter_hot_pos() {
            if self.is_valid_double_three::<false>(SetOverrideStack::new(double_three_pos), Direction::Vertical, double_three_pos) {
                self.patterns.field[double_three_pos.idx_usize()].black_unit.unmark_invalid_double_three();
            } else {
                self.patterns.field[double_three_pos.idx_usize()].black_unit.mark_invalid_double_three();
            }
        }
    }

    #[cfg(not(feature = "strict_renju"))]
    #[inline(always)]
    fn is_invalid_three_component<const IS_NESTED: bool>(&self, _overrides: SetOverrideStack, _from_direction: Direction, pos: Pos) -> bool {
        let pattern_unit = self.patterns.field[pos.idx_usize()].black_unit;

        !pattern_unit.has_three() // non-three
            || pattern_unit.has_four() // double-four
            || pattern_unit.has_overline() // overline
    }

    #[cfg(feature = "strict_renju")]
    #[inline(always)]
    fn is_invalid_three_component<const IS_NESTED: bool>(&self, overrides: SetOverrideStack, from_direction: Direction, pos: Pos) -> bool {
        let pattern_unit = self.patterns.field[pos.idx_usize()].black_unit;

        if !pattern_unit.has_three() // non-three
            || pattern_unit.has_four() || overrides.four_contains(&pos) // double-four
            || pattern_unit.has_overline() // overline
            || overrides.set_contains(&pos) // recursive
        {
            return true;
        }

        // nested double-three
        pattern_unit.count_open_threes() > 2 && {
            let mut new_overrides = overrides;
            if !IS_NESTED {
                self.update_four_overrides_root(&mut new_overrides);
            }
            self.update_four_overrides(&mut new_overrides, from_direction, pos);

            self.is_valid_double_three::<true>(new_overrides, from_direction, pos)
        }
    }

    fn is_valid_double_three<const IS_NESTED: bool>(&self, overrides: SetOverrideStack, from_direction: Direction, pos: Pos) -> bool {
        let pattern_unit = self.patterns.field[pos.idx_usize()].black_unit;
        let mut total_threes = if IS_NESTED {
            pattern_unit.count_open_threes() - 1
        } else {
            pattern_unit.count_open_threes()
        };

        for direction in pattern_unit.iter_three_directions() {
            if IS_NESTED && direction == from_direction {
                continue;
            }

            if match self.calculate_three_signature(direction, pos) {
                /* .VOO. */ 0b11000 => {
                    self.is_invalid_three_component::<IS_NESTED>(overrides, direction, pos.directional_negative_offset(direction, 1)) &&
                    self.is_invalid_three_component::<IS_NESTED>(overrides, direction, pos.directional_positive_offset(direction, 3))
                },
                /* .OOV. */ 0b00011 => {
                    self.is_invalid_three_component::<IS_NESTED>(overrides, direction, pos.directional_negative_offset(direction, 3)) &&
                    self.is_invalid_three_component::<IS_NESTED>(overrides, direction, pos.directional_positive_offset(direction, 1))
                },
                /* V.OO  */ 0b10000 => {
                    self.is_invalid_three_component::<IS_NESTED>(overrides, direction, pos.directional_positive_offset(direction, 1))
                }
                /* OO.V  */ 0b00001 => {
                    self.is_invalid_three_component::<IS_NESTED>(overrides, direction, pos.directional_negative_offset(direction, 1))
                }
                /* VO.O  */ 0b01000 => {
                    self.is_invalid_three_component::<IS_NESTED>(overrides, direction, pos.directional_positive_offset(direction, 2))
                },
                /* .OVO. */ 0b01010 => {
                    self.is_invalid_three_component::<IS_NESTED>(overrides, direction, pos.directional_negative_offset(direction, 2)) &&
                    self.is_invalid_three_component::<IS_NESTED>(overrides, direction, pos.directional_positive_offset(direction, 2))
                },
                /* O.OV  */ 0b00010 => {
                    self.is_invalid_three_component::<IS_NESTED>(overrides, direction, pos.directional_negative_offset(direction, 2))
                },
                /* OV.O  */ 0b10010 => {
                    self.is_invalid_three_component::<IS_NESTED>(overrides, direction, pos.directional_positive_offset(direction, 1))
                },
                /* O.VO  */ 0b01001 => {
                    self.is_invalid_three_component::<IS_NESTED>(overrides, direction, pos.directional_negative_offset(direction, 1))
                },
                _ => unreachable!()
            } {
                if total_threes < 3 {
                    return false;
                }
                total_threes -= 1;
            }
        }

        true
    }

    fn update_four_overrides_root(&self, overrides: &mut SetOverrideStack) {
        let pos = overrides.set[0];

        for direction in self.patterns.field[pos.idx_usize()].black_unit.iter_three_directions() {
            self.update_four_overrides_each_direction(overrides, direction, pos);
        }
    }

    fn update_four_overrides(&self, overrides: &mut SetOverrideStack, from_direction: Direction, pos: Pos) {
        for next_four_idx in 0 .. 12 {
            if next_four_idx / 3 != from_direction as u8 {
                let four_pos = overrides.next_four[next_four_idx as usize];
                if four_pos != INVALID_POS {
                    overrides.four[overrides.four_top as usize] = four_pos;
                    overrides.four_top += 1;
                }
            }
        }

        overrides.next_four = [INVALID_POS; 12];

        for direction in self.patterns.field[pos.idx_usize()].black_unit.iter_three_directions() {
            if direction == from_direction {
                continue;
            }

            self.update_four_overrides_each_direction(overrides, direction, pos);
        }

        overrides.set[overrides.set_top as usize] = pos;
        overrides.set_top += 1;
    }

    #[inline(always)]
    fn update_four_overrides_each_direction(&self, overrides: &mut SetOverrideStack, direction: Direction, pos: Pos) {
        let direction_offset = direction as usize * 3;

        match self.calculate_three_signature(direction, pos) {
            /* .VOO.  */ 0b11000 => {
                overrides.next_four[direction_offset] = pos.directional_negative_offset(direction, 1);
                overrides.next_four[direction_offset + 1] = pos.directional_positive_offset(direction, 3);
            },
            /* .OOV.  */ 0b00011 => {
                overrides.next_four[direction_offset] = pos.directional_negative_offset(direction, 3);
                overrides.next_four[direction_offset + 1] = pos.directional_positive_offset(direction, 1);
            },
            /* .V.OO. */ 0b10000 => {
                overrides.next_four[direction_offset] = pos.directional_negative_offset(direction, 1);
                overrides.next_four[direction_offset + 1] = pos.directional_positive_offset(direction, 1);
                overrides.next_four[direction_offset + 2] = pos.directional_positive_offset(direction, 3);
            }
            /* .OO.V. */ 0b00001 => {
                overrides.next_four[direction_offset] = pos.directional_negative_offset(direction, 3);
                overrides.next_four[direction_offset + 1] = pos.directional_negative_offset(direction, 1);
                overrides.next_four[direction_offset + 2] = pos.directional_positive_offset(direction, 1);
            }
            /* .VO.O. */ 0b01000 => {
                overrides.next_four[direction_offset] = pos.directional_negative_offset(direction, 1);
                overrides.next_four[direction_offset + 1] = pos.directional_positive_offset(direction, 2);
                overrides.next_four[direction_offset + 2] = pos.directional_positive_offset(direction, 4);
            },
            /* .OVO.  */ 0b01010 => {
                overrides.next_four[direction_offset] = pos.directional_negative_offset(direction, 2);
                overrides.next_four[direction_offset + 1] = pos.directional_positive_offset(direction, 2);
            },
            /* .O.OV. */ 0b00010 => {
                overrides.next_four[direction_offset] = pos.directional_negative_offset(direction, 4);
                overrides.next_four[direction_offset + 1] = pos.directional_negative_offset(direction, 2);
                overrides.next_four[direction_offset + 2] = pos.directional_positive_offset(direction, 1);
            },
            /* .OV.O. */ 0b10010 => {
                overrides.next_four[direction_offset] = pos.directional_negative_offset(direction, 2);
                overrides.next_four[direction_offset + 1] = pos.directional_positive_offset(direction, 1);
                overrides.next_four[direction_offset + 2] = pos.directional_positive_offset(direction, 3);
            },
            /* .O.VO. */ 0b01001 => {
                overrides.next_four[direction_offset] = pos.directional_negative_offset(direction, 3);
                overrides.next_four[direction_offset + 1] = pos.directional_negative_offset(direction, 1);
                overrides.next_four[direction_offset + 2] = pos.directional_positive_offset(direction, 2);
            },
            _ => unreachable!()
        }
    }

    fn calculate_three_signature(&self, direction: Direction, pos: Pos) -> u16 {
        let slice = self.slices.access_slice(direction, pos);
        let slice_idx = slice.calculate_idx(direction, pos);
        (slice.black_stones >> (slice_idx - 2)) & 0b11111 // 0[00V00]0
    }

}

// 48 bytes
#[derive(Copy, Clone)]
struct SetOverrideStack {
    set: [Pos; 7],
    set_top: u8,
    four: [Pos; 27],
    four_top: u8,
    next_four: [Pos; 12],
}

impl SetOverrideStack {

    fn set_contains(&self, pos: &Pos) -> bool {
        for idx in 0 .. self.set_top {
            if &self.set[idx as usize] == pos {
                return true;
            }
        }

        false
    }

    fn four_contains(&self, pos: &Pos) -> bool {
        for idx in 0 .. self.four_top {
            if &self.four[idx as usize] == pos {
                return true;
            }
        }

        false
    }

    fn new(root: Pos) -> Self {
        Self {
            set: [root, INVALID_POS, INVALID_POS, INVALID_POS, INVALID_POS, INVALID_POS, INVALID_POS],
            set_top: 1,
            four: [INVALID_POS; 27],
            four_top: 0,
            next_four: [INVALID_POS; 12],
        }
    }

}
