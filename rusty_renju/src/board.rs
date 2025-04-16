use crate::assert_struct_sizes;
use crate::bitfield::Bitfield;
use crate::memo::hash_key::HashKey;
use crate::notation::color::Color;
use crate::notation::direction::Direction;
use crate::notation::pos::{MaybePos, Pos};
use crate::pattern;
use crate::pattern::Patterns;
use crate::slice::Slices;
use std::marker::ConstParamTy;

#[derive(Copy, Clone, Default)]
pub struct Board {
    pub player_color: Color,
    pub stones: u8,
    pub slices: Slices,
    pub patterns: Patterns,
    pub hot_field: Bitfield,
    pub hash_key: HashKey,
}

impl Board {

    pub const fn opponent_color(&self) -> Color {
        self.player_color.reversed()
    }

    pub fn is_pos_empty(&self, pos: Pos) -> bool {
        self.slices.horizontal_slices[pos.row_usize()].is_empty(pos.col())
    }

    pub fn stone_kind(&self, pos: Pos) -> Option<Color> {
        self.slices.horizontal_slices[pos.row_usize()].stone_kind(pos.col())
    }

    pub fn set(mut self, pos: Pos) -> Self {
        self.set_mut(pos);
        self
    }

    pub fn unset(mut self, pos: Pos) -> Self {
        self.unset_mut(pos);
        self
    }

    pub fn pass(mut self) -> Self {
        self.pass_mut();
        self
    }

    pub fn set_mut(&mut self, pos: Pos) {
        self.stones += 1;
        self.hot_field.set_mut(pos);
        self.hash_key = self.hash_key.set(self.player_color, pos);

        self.incremental_update_mut::<{ MoveType::Set }>(pos);

        self.switch_player_mut();
    }

    pub fn unset_mut(&mut self, pos: Pos) {
        self.patterns.unchecked_five_in_a_row = None;

        self.switch_player_mut();

        self.stones -= 1;
        self.hot_field.unset_mut(pos);
        self.hash_key = self.hash_key.set(self.player_color, pos);

        self.incremental_update_mut::<{ MoveType::Unset }>(pos);
    }

    pub fn pass_mut(&mut self) {
        self.switch_player_mut();
    }

    pub fn batch_set_mut(&mut self, moves: &[Pos]) {
        let odd_moves = moves.iter()
            .enumerate()
            .filter_map(|(idx, &pos)| (idx % 2 == 1).then_some(pos))
            .collect::<Vec<_>>();

        let even_moves = moves.iter()
            .enumerate()
            .filter_map(|(idx, &pos)| (idx % 2 == 0).then_some(pos))
            .collect::<Vec<_>>();

        let (black_moves, white_moves) = match self.player_color {
            Color::Black => (even_moves, odd_moves),
            Color::White => (odd_moves, even_moves)
        };

        let player = Color::player_color_from_each_moves(black_moves.len(), white_moves.len());

        self.batch_set_each_color_mut(black_moves.into_boxed_slice(), white_moves.into_boxed_slice(), player)
    }

    pub fn batch_set_each_color_mut(&mut self, blacks: Box<[Pos]>, whites: Box<[Pos]>, player: Color) {
        self.stones += blacks.len() as u8 + whites.len() as u8;

        for pos in blacks {
            self.slices.set_mut(Color::Black, pos);
            self.hot_field.set_mut(pos);
        }

        for pos in whites {
            self.slices.set_mut(Color::White, pos);
            self.hot_field.set_mut(pos);
        }

        self.player_color = player;

        self.full_update_mut();
        self.hash_key = HashKey::from(&self.slices.horizontal_slices);
    }

    pub fn switch_player_mut(&mut self) {
        self.player_color = self.opponent_color();
    }

    fn incremental_update_mut<const M: MoveType>(&mut self, pos: Pos) {
        macro_rules! update_by_slice {
            ($direction:expr,$slice:expr,$slice_idx:expr) => {{
                match M {
                    MoveType::Set => $slice.set_mut(self.player_color, $slice_idx),
                    MoveType::Unset => $slice.unset_mut(self.player_color, $slice_idx)
                }

                match (
                    *$slice.pattern_bitmap.player_unit::<{ Color::Black }>() == 0,
                    $slice.has_potential_pattern::<{ Color::Black }>()
                ) {
                    (_, true) => {
                        self.patterns.update_with_slice_mut::<{ Color::Black }, { $direction }>($slice);
                    },
                    (false, false) => {
                        self.patterns.clear_with_slice_mut::<{ Color::Black }, { $direction }>($slice);
                    },
                    _ => {}
                }

                match (
                    *$slice.pattern_bitmap.player_unit::<{ Color::White }>() == 0,
                    $slice.has_potential_pattern::<{ Color::White }>()
                ) {
                    (_, true) => {
                        self.patterns.update_with_slice_mut::<{ Color::White }, { $direction }>($slice);
                    },
                    (false, false) => {
                        self.patterns.clear_with_slice_mut::<{ Color::White }, { $direction }>($slice);
                    },
                    _ => {}
                }
            }};
        }

        self.patterns.unchecked_five_pos = Patterns::EMPTY_UNCHECKED_FIVE_POS;

        let horizontal_slice = &mut self.slices.horizontal_slices[pos.row_usize()];
        update_by_slice!(Direction::Horizontal, horizontal_slice, pos.col());

        let vertical_slice = &mut self.slices.vertical_slices[pos.col_usize()];
        update_by_slice!(Direction::Vertical, vertical_slice, pos.row());

        if let Some(ascending_slice_idx) = Slices::ascending_slice_idx(pos) {
            let ascending_slice = &mut self.slices.ascending_slices[ascending_slice_idx];
            update_by_slice!(Direction::Ascending, ascending_slice, pos.col() - ascending_slice.start_col);
        }

        if let Some(descending_slice_idx) = Slices::descending_slice_idx(pos) {
            let descending_slice = &mut self.slices.descending_slices[descending_slice_idx];
            update_by_slice!(Direction::Descending, descending_slice, pos.col() - descending_slice.start_col);
        }

        self.validate_double_three_mut();
    }

    fn full_update_mut(&mut self) {
        macro_rules! update_by_slice {
            ($slice:expr,$direction:expr) => {{
                if $slice.has_potential_pattern::<{ Color::Black }>() {
                    self.patterns.update_with_slice_mut::<{ Color::Black }, { $direction }>($slice);
                }

                if $slice.has_potential_pattern::<{ Color::White }>() {
                    self.patterns.update_with_slice_mut::<{ Color::White }, { $direction }>($slice);
                }
            }};
        }

        for horizontal_slice in self.slices.horizontal_slices.iter_mut() {
            update_by_slice!(horizontal_slice, Direction::Horizontal);
        }

        for vertical_slice in self.slices.vertical_slices.iter_mut() {
            update_by_slice!(vertical_slice, Direction::Vertical);
        }

        for ascending_slice in self.slices.ascending_slices.iter_mut() {
            update_by_slice!(ascending_slice, Direction::Ascending);
        }

        for descending_slice in self.slices.descending_slices.iter_mut() {
            update_by_slice!(descending_slice, Direction::Descending);
        }

        self.validate_double_three_mut();
    }

    fn validate_double_three_mut(&mut self) {
        for root_pos in self.patterns.unchecked_double_three_field.iter_hot_pos() {
            if self.is_valid_double_three(ValidateThreeRoot { root_pos }) {
                self.patterns.field.black[root_pos.idx_usize()].unmark_invalid_double_three();
            } else {
                self.patterns.field.black[root_pos.idx_usize()].mark_invalid_double_three();
            }
        }
    }

    #[cfg(not(feature = "strict_renju"))]
    fn is_invalid_three_component<C: ValidateThreeContext>(&self, context: C, direction: Direction, offset: isize) -> bool {
        const ANY_FOUR_OR_OVERLINE_MASK: u32 = pattern::UNIT_ANY_FOUR_MASK | pattern::UNIT_OVERLINE_MASK;

        let pos = context.parent_pos().directional_offset_unchecked(direction, offset);

        let pattern = self.patterns.field.black[pos.idx_usize()];

        !pattern.has_three() // non-three
            || pattern.apply_mask(ANY_FOUR_OR_OVERLINE_MASK) != 0 // double-four or overline
            || pattern.count_open_threes() > 2 // maybe nested double-three
    }

    #[cfg(feature = "strict_renju")]
    fn is_invalid_three_component<C: ValidateThreeContext>(&self, context: C, direction: Direction, offset: isize) -> bool {
        const ANY_FOUR_OR_OVERLINE_MASK: u32 = pattern::UNIT_ANY_FOUR_MASK | pattern::UNIT_OVERLINE_MASK;

        let pos = context.parent_pos().directional_offset_unchecked(direction, offset);

        let pattern = self.patterns.field.black[pos.idx_usize()];

        !pattern.has_three() // non-three
            || pattern.apply_mask(ANY_FOUR_OR_OVERLINE_MASK) != 0 // double-four or overline
            || context.four_contains(pos) // double-four
            || context.set_contains(pos) // recursive
            || (pattern.count_open_threes() > 2 && { // nested double-three
                let mut new_overrides = context.branch_overrides();

                if C::IS_ROOT {
                    self.update_root_four_overrides(&mut new_overrides);
                }

                self.update_four_overrides(&mut new_overrides, direction, pos);

                self.is_valid_double_three(ValidateThreeNode {
                    overrides: new_overrides,
                    parent_direction: direction,
                    parent_pos: pos,
                })
            })
    }

    fn is_valid_double_three<C: ValidateThreeContext>(&self, context: C) -> bool {
        let pos = context.parent_pos();
        let pattern_unit = self.patterns.field.black[pos.idx_usize()];

        let mut total_threes = if C::IS_ROOT {
            pattern_unit.count_open_threes()
        } else {
            pattern_unit.count_open_threes() - 1
        };

        for direction in pattern_unit.iter_three_directions() {
            if !C::IS_ROOT && direction == context.parent_direction() {
                continue;
            }

            if match self.calculate_near_four_window::<{ Color::Black }>(direction, pos) {
                /* .VOO. */ 0b11000 => {
                    self.is_invalid_three_component(context, direction, -1) &&
                    self.is_invalid_three_component(context, direction, 3)
                },
                /* .OOV. */ 0b00011 => {
                    self.is_invalid_three_component(context, direction, -3) &&
                    self.is_invalid_three_component(context, direction, 1)
                },
                /* V.OO  */ 0b10000 => {
                    self.is_invalid_three_component(context, direction, 1)
                }
                /* OO.V  */ 0b00001 => {
                    self.is_invalid_three_component(context, direction, -1)
                }
                /* VO.O  */ 0b01000 => {
                    self.is_invalid_three_component(context, direction, 2)
                },
                /* .OVO. */ 0b01010 => {
                    self.is_invalid_three_component(context, direction, -2) &&
                    self.is_invalid_three_component(context, direction, 2)
                },
                /* O.OV  */ 0b00010 => {
                    self.is_invalid_three_component(context, direction, -2)
                },
                /* OV.O  */ 0b10010 => {
                    self.is_invalid_three_component(context, direction, 1)
                },
                /* O.VO  */ 0b01001 => {
                    self.is_invalid_three_component(context, direction, -1)
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

    fn update_root_four_overrides(&self, overrides: &mut SetOverrides) {
        let pos = overrides.set[0];

        for direction in self.patterns.field.black[pos.idx_usize()].iter_three_directions() {
            self.update_four_overrides_each_direction(overrides, direction, pos);
        }
    }

    fn update_four_overrides(&self, overrides: &mut SetOverrides, direction_from: Direction, pos: Pos) {
        for next_four_idx in (0 .. direction_from as u8 * 3).chain(direction_from as u8 * 4 .. 12) {
            let four_pos = overrides.next_four[next_four_idx as usize];
            if four_pos != MaybePos::NONE.unwrap() {
                overrides.four[overrides.four_top as usize] = four_pos;
                overrides.four_top += 1;
            }
        }

        overrides.next_four = [MaybePos::NONE.unwrap(); 12];

        for direction in self.patterns.field.black[pos.idx_usize()].iter_three_directions() {
            if direction == direction_from {
                continue;
            }

            self.update_four_overrides_each_direction(overrides, direction, pos);
        }

        overrides.set[overrides.set_top as usize] = pos;
        overrides.set_top += 1;
    }

    fn update_four_overrides_each_direction(&self, overrides: &mut SetOverrides, direction: Direction, pos: Pos) {
        let direction_offset = direction as usize * 3;

        match self.calculate_near_four_window::<{ Color::Black }>(direction, pos) {
            /* .VOO.  */ 0b11000 => {
                overrides.next_four[direction_offset] = pos.directional_offset_unchecked(direction, -1);
                overrides.next_four[direction_offset + 1] = pos.directional_offset_unchecked(direction, 3);
            },
            /* .OOV.  */ 0b00011 => {
                overrides.next_four[direction_offset] = pos.directional_offset_unchecked(direction, -3);
                overrides.next_four[direction_offset + 1] = pos.directional_offset_unchecked(direction, 1);
            },
            /* .V.OO. */ 0b10000 => {
                overrides.next_four[direction_offset] = pos.directional_offset_unchecked(direction, -1);
                overrides.next_four[direction_offset + 1] = pos.directional_offset_unchecked(direction, 1);
                overrides.next_four[direction_offset + 2] = pos.directional_offset_unchecked(direction, 3);
            }
            /* .OO.V. */ 0b00001 => {
                overrides.next_four[direction_offset] = pos.directional_offset_unchecked(direction, -3);
                overrides.next_four[direction_offset + 1] = pos.directional_offset_unchecked(direction, -1);
                overrides.next_four[direction_offset + 2] = pos.directional_offset_unchecked(direction, 1);
            }
            /* .VO.O. */ 0b01000 => {
                overrides.next_four[direction_offset] = pos.directional_offset_unchecked(direction, -1);
                overrides.next_four[direction_offset + 1] = pos.directional_offset_unchecked(direction, 2);
                overrides.next_four[direction_offset + 2] = pos.directional_offset_unchecked(direction, 4);
            },
            /* .OVO.  */ 0b01010 => {
                overrides.next_four[direction_offset] = pos.directional_offset_unchecked(direction, -2);
                overrides.next_four[direction_offset + 1] = pos.directional_offset_unchecked(direction, 2);
            },
            /* .O.OV. */ 0b00010 => {
                overrides.next_four[direction_offset] = pos.directional_offset_unchecked(direction, -4);
                overrides.next_four[direction_offset + 1] = pos.directional_offset_unchecked(direction, -2);
                overrides.next_four[direction_offset + 2] = pos.directional_offset_unchecked(direction, 1);
            },
            /* .OV.O. */ 0b10010 => {
                overrides.next_four[direction_offset] = pos.directional_offset_unchecked(direction, -2);
                overrides.next_four[direction_offset + 1] = pos.directional_offset_unchecked(direction, 1);
                overrides.next_four[direction_offset + 2] = pos.directional_offset_unchecked(direction, 3);
            },
            /* .O.VO. */ 0b01001 => {
                overrides.next_four[direction_offset] = pos.directional_offset_unchecked(direction, -3);
                overrides.next_four[direction_offset + 1] = pos.directional_offset_unchecked(direction, -1);
                overrides.next_four[direction_offset + 2] = pos.directional_offset_unchecked(direction, 2);
            },
            _ => unreachable!()
        }
    }

    fn calculate_near_four_window<const C: Color>(&self, direction: Direction, pos: Pos) -> u8 {
        let slice = self.slices.access_slice_unchecked(direction, pos);
        let slice_idx = slice.calculate_slice_idx(direction, pos);

        let stones = match C {
            Color::Black => slice.black_stones,
            Color::White => slice.white_stones
        } as u32;

        (((stones << 2) >> slice_idx) & 0b11111) as u8 // 0[00V00]0
    }

}

//noinspection RsUnresolvedPath
#[derive(ConstParamTy, Eq, PartialEq,)]
enum MoveType {
    Set, Unset
}

trait ValidateThreeContext : Copy {

    const IS_ROOT: bool;

    fn parent_pos(&self) -> Pos;

    fn parent_direction(&self) -> Direction;

    fn branch_overrides(&self) -> SetOverrides;

    fn set_contains(&self, pos: Pos) -> bool;

    fn four_contains(&self, pos: Pos) -> bool;

}

#[derive(Copy, Clone)]
struct ValidateThreeRoot {
    root_pos: Pos
}

impl ValidateThreeContext for ValidateThreeRoot {
    const IS_ROOT: bool = true;

    fn parent_pos(&self) -> Pos {
        self.root_pos
    }

    fn parent_direction(&self) -> Direction {
        unreachable!()
    }

    fn branch_overrides(&self) -> SetOverrides {
        SetOverrides::new(self.root_pos)
    }

    fn set_contains(&self, _pos: Pos) -> bool {
        false
    }

    fn four_contains(&self, _pos: Pos) -> bool {
        false
    }
}

#[derive(Copy, Clone)]
#[repr(align(8))]
struct ValidateThreeNode {
    overrides: SetOverrides,
    parent_direction: Direction,
    parent_pos: Pos,
}

impl ValidateThreeContext for ValidateThreeNode {
    const IS_ROOT: bool = false;

    fn parent_pos(&self) -> Pos {
        self.parent_pos
    }

    fn parent_direction(&self) -> Direction {
        self.parent_direction
    }

    fn branch_overrides(&self) -> SetOverrides {
        self.overrides
    }

    fn set_contains(&self, pos: Pos) -> bool {
        self.overrides.set[..self.overrides.set_top as usize].contains(&pos)
    }

    fn four_contains(&self, pos: Pos) -> bool {
        self.overrides.four[..self.overrides.four_top as usize].contains(&pos)
    }
}

#[derive(Copy, Clone)]
#[repr(align(8))]
pub struct SetOverrides {
    set: [Pos; 6],
    set_top: u8,
    four: [Pos; 20],
    four_top: u8,
    next_four: [Pos; 12],
}

assert_struct_sizes!(SetOverrides, size=40, align=8);

impl SetOverrides {

    fn new(root: Pos) -> Self {
        Self {
            set: {
                const SET: [Pos; 6] = [MaybePos::NONE.unwrap(); 6];
                let mut set = SET;
                set[0] = root;
                set
            },
            set_top: 1,
            four: [MaybePos::NONE.unwrap(); 20],
            four_top: 0,
            next_four: [MaybePos::NONE.unwrap(); 12],
        }
    }

}
