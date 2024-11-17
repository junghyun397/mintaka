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

    #[inline(always)]
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

        let mut packed_slices: [u64; 3] = [0, 0, 0];
        let mut prefetch_idx = 0;

        if is_vertical_slice_valid {
            packed_slices[prefetch_idx] = vertical_slice.packed_slice();
            prefetch_idx += 1;
        }

        if valid_ascending_slice.as_ref()
            .map_or(false, |slice| slice.is_valid_pattern())
        {
            packed_slices[prefetch_idx] = valid_ascending_slice.as_ref().unwrap().packed_slice();
            prefetch_idx += 1;
        }

        if valid_descending_slice.as_ref()
            .map_or(false, |slice| slice.is_valid_pattern())
        {
            packed_slices[prefetch_idx] = valid_descending_slice.as_ref().unwrap().packed_slice();
        }

        prefetch_idx = 0;

        if is_horizontal_slice_valid {
            if packed_slices[prefetch_idx] != 0 {
                memo.prefetch_memo(packed_slices[prefetch_idx]);
                prefetch_idx += 1;
            }

            self.patterns.update_by_slice_mut::<{ Direction::Horizontal }>(memo, horizontal_slice);
        }

        if is_vertical_slice_valid {
            if packed_slices[prefetch_idx] != 0 {
                memo.prefetch_memo(packed_slices[prefetch_idx]);
                prefetch_idx += 1;
            }

            self.patterns.update_by_slice_mut::<{ Direction::Vertical }>(memo, vertical_slice);
        }

        if let Some(ascending_slice) = valid_ascending_slice {
            if packed_slices[prefetch_idx] != 0 {
                memo.prefetch_memo(packed_slices[prefetch_idx]);
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
                self.patterns.field[double_three_pos.idx_usize()].black_unit.mark_valid_double_three();
            } else {
                self.patterns.field[double_three_pos.idx_usize()].black_unit.unmark_valid_double_three();
            }
        }
    }

    #[inline(always)]
    fn is_invalid_three_component(&self, stack: SetOverrideStack, from_direction: Direction, pos: Pos) -> bool {
        let pattern_unit = self.patterns.field[pos.idx_usize()].black_unit;

        if !pattern_unit.has_three() // non-three
            || pattern_unit.has_four() || stack.four.contains(&pos) // double-four
            || pattern_unit.has_overline() // overline
            || stack.set.contains(&pos) // recursive
        {
            return true;
        }

        // nested double-three
        if pattern_unit.count_open_threes() > 2 {
            let pattern_unit = self.patterns.field[pos.idx_usize()].black_unit;

            let mut new_stack = stack;
            
            for next_four_idx in 0 .. new_stack.next_four_top {
                new_stack.four[(new_stack.four_top + next_four_idx) as usize] = stack.next_four[next_four_idx as usize];
            }
            new_stack.four_top += stack.next_four_top;
            new_stack.next_four = [INVALID_POS; 9];
            new_stack.next_four_top = 0;
            
            for direction in pattern_unit.iter_three_directions() {
                if direction == from_direction {
                    continue;
                }

                let signature = {
                    let slice = self.slices.access_slice(direction, pos);
                    let slice_idx = slice.calculate_idx(direction, pos);
                    (slice.black_stones >> (slice_idx - 2)) & 0b11111 // 0[00V00]0
                };

                match signature {
                    /* .VOO. */ 0b11000 => {
                        new_stack.next_four[new_stack.next_four_top as usize] = pos.directional_negative_offset(direction, 1);
                        new_stack.next_four[new_stack.next_four_top as usize + 1] = pos.directional_positive_offset(direction, 3);
                        new_stack.next_four_top += 2;
                    },
                    /* .OOV. */ 0b00011 => {
                        new_stack.next_four[new_stack.next_four_top as usize] = pos.directional_negative_offset(direction, 3);
                        new_stack.next_four[new_stack.next_four_top as usize + 1] = pos.directional_positive_offset(direction, 1);
                        new_stack.next_four_top += 2;
                    },
                    /* V.OO  */ 0b10000 => {
                        new_stack.next_four[new_stack.next_four_top as usize] = pos.directional_negative_offset(direction, 1);
                        new_stack.next_four[new_stack.next_four_top as usize + 1] = pos.directional_positive_offset(direction, 1);
                        new_stack.next_four[new_stack.next_four_top as usize + 2] = pos.directional_positive_offset(direction, 3);
                        new_stack.next_four_top += 3;
                    }
                    /* OO.V  */ 0b00001 => {
                        new_stack.next_four[new_stack.next_four_top as usize] = pos.directional_negative_offset(direction, 1);
                        new_stack.next_four[new_stack.next_four_top as usize + 1] = pos.directional_negative_offset(direction, 3);
                        new_stack.next_four[new_stack.next_four_top as usize + 2] = pos.directional_positive_offset(direction, 1);
                        new_stack.next_four_top += 3;
                    }
                    /* VO.O  */ 0b01000 => {
                        new_stack.next_four[new_stack.next_four_top as usize] = pos.directional_negative_offset(direction, 1);
                        new_stack.next_four[new_stack.next_four_top as usize + 1] = pos.directional_positive_offset(direction, 2);
                        new_stack.next_four[new_stack.next_four_top as usize + 2] = pos.directional_positive_offset(direction, 4);
                        new_stack.next_four_top += 3;
                    },
                    /* .OVO. */ 0b01010 => {
                        new_stack.next_four[new_stack.next_four_top as usize] = pos.directional_negative_offset(direction, 2);
                        new_stack.next_four[new_stack.next_four_top as usize + 1] = pos.directional_positive_offset(direction, 2);
                        new_stack.next_four_top += 2;
                    },
                    /* O.OV  */ 0b00010 => {
                        new_stack.next_four[new_stack.next_four_top as usize] = pos.directional_negative_offset(direction, 4);
                        new_stack.next_four[new_stack.next_four_top as usize + 1] = pos.directional_negative_offset(direction, 2);
                        new_stack.next_four[new_stack.next_four_top as usize + 2] = pos.directional_positive_offset(direction, 1);
                        new_stack.next_four_top += 3;
                    },
                    /* OV.O  */ 0b10010 => {
                        new_stack.next_four[new_stack.next_four_top as usize] = pos.directional_negative_offset(direction, 2);
                        new_stack.next_four[new_stack.next_four_top as usize + 1] = pos.directional_positive_offset(direction, 1);
                        new_stack.next_four[new_stack.next_four_top as usize + 2] = pos.directional_positive_offset(direction, 3);
                        new_stack.next_four_top += 3;
                    },
                    /* O.VO  */ 0b01001 => {
                        new_stack.next_four[new_stack.next_four_top as usize] = pos.directional_negative_offset(direction, 3);
                        new_stack.next_four[new_stack.next_four_top as usize + 1] = pos.directional_negative_offset(direction, 1);
                        new_stack.next_four[new_stack.next_four_top as usize + 2] = pos.directional_positive_offset(direction, 2);
                        new_stack.next_four_top += 3;
                    },
                    _ => unreachable!()
                }
            }
            
            new_stack.set[new_stack.set_top as usize] = pos;
            new_stack.set_top += 1;

            return self.is_valid_double_three::<true>(new_stack, from_direction, pos)
        }
        
        false
    }

    fn is_valid_double_three<const IS_NESTED: bool>(&self, stack: SetOverrideStack, from_direction: Direction, pos: Pos) -> bool {
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
            
            let signature = {
                let slice = self.slices.access_slice(direction, pos);
                let slice_idx = slice.calculate_idx(direction, pos);
                (slice.black_stones >> (slice_idx - 2)) & 0b11111 // 0[00V00]0
            };

            if match signature {
                /* .VOO. */ 0b11000 => {
                    self.is_invalid_three_component(stack, direction, pos.directional_negative_offset(direction, 1)) &&
                    self.is_invalid_three_component(stack, direction, pos.directional_positive_offset(direction, 3))
                },
                /* .OOV. */ 0b00011 => {
                    self.is_invalid_three_component(stack, direction, pos.directional_negative_offset(direction, 3)) &&
                    self.is_invalid_three_component(stack, direction, pos.directional_positive_offset(direction, 1))
                },
                /* V.OO  */ 0b10000 => {
                    self.is_invalid_three_component(stack, direction, pos.directional_positive_offset(direction, 1))
                }
                /* OO.V  */ 0b00001 => {
                    self.is_invalid_three_component(stack, direction, pos.directional_negative_offset(direction, 1))
                }
                /* VO.O  */ 0b01000 => {
                    self.is_invalid_three_component(stack, direction, pos.directional_positive_offset(direction, 2))
                },
                /* .OVO. */ 0b01010 => {
                    self.is_invalid_three_component(stack, direction, pos.directional_negative_offset(direction, 2)) &&
                    self.is_invalid_three_component(stack, direction, pos.directional_positive_offset(direction, 2))
                },
                /* O.OV  */ 0b00010 => {
                    self.is_invalid_three_component(stack, direction, pos.directional_negative_offset(direction, 2))
                },
                /* OV.O  */ 0b10010 => {
                    self.is_invalid_three_component(stack, direction, pos.directional_positive_offset(direction, 1))
                },
                /* O.VO  */ 0b01001 => {
                    self.is_invalid_three_component(stack, direction, pos.directional_negative_offset(direction, 1))
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

}

#[derive(Copy, Clone)]
struct SetOverrideStack {
    set: [Pos; 7],
    set_top: u8,
    four: [Pos; 45],
    four_top: u8,
    next_four: [Pos; 9],
    next_four_top: u8
}

impl SetOverrideStack {
    
    fn new(root: Pos) -> Self {
        Self {
            set: [root, INVALID_POS, INVALID_POS, INVALID_POS, INVALID_POS, INVALID_POS, INVALID_POS],
            set_top: 1,
            four: [INVALID_POS; 45],
            four_top: 0,
            next_four: [INVALID_POS; 9],
            next_four_top: 0,
        }       
    }

}
