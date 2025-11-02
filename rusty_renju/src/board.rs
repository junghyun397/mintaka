use crate::bitfield::Bitfield;
use crate::memo::hash_key::HashKey;
use crate::notation::color::Color;
use crate::notation::direction::Direction;
use crate::notation::pos::{MaybePos, Pos};
use crate::notation::rule::RuleKind;
use crate::pattern;
use crate::pattern::Patterns;
use crate::slice::{Slice, Slices};

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

    pub fn is_pos_empty(&self, pos: Pos) -> bool {
        self.hot_field.is_cold(pos)
    }

    pub fn is_legal_move(&self, pos: Pos) -> bool {
        self.is_pos_empty(pos)
            && (self.player_color != Color::Black || !self.patterns.is_forbidden(pos))
    }

    pub fn legal_field(&self) -> Bitfield {
        match self.player_color {
            Color::Black => !(self.hot_field | self.patterns.forbidden_field),
            Color::White => !self.hot_field
        }
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
        self.hot_field.set(pos);
        self.hash_key = self.hash_key.set(self.player_color, pos);

        self.incremental_update_mut::<{ MoveType::Set }>(pos);

        self.switch_player_mut();
    }

    pub fn unset_mut(&mut self, pos: Pos) {
        self.switch_player_mut();

        self.stones -= 1;
        self.hot_field.unset(pos);
        self.hash_key = self.hash_key.set(self.player_color, pos);

        self.incremental_update_mut::<{ MoveType::Unset }>(pos);
    }

    pub fn pass_mut(&mut self) {
        self.switch_player_mut();
    }

    pub fn unpass_mut(&mut self) {
        self.switch_player_mut();
    }

    pub fn batch_set_mut(&mut self, moves: &[MaybePos]) {
        let odd_moves = moves.iter()
            .enumerate()
            .filter_map(|(idx, &pos)|
                (!idx.is_multiple_of(2)).then_some(pos).and_then(MaybePos::into)
            )
            .collect::<Vec<_>>();

        let even_moves = moves.iter()
            .enumerate()
            .filter_map(|(idx, &pos)|
                idx.is_multiple_of(2).then_some(pos).and_then(MaybePos::into)
            )
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
            self.slices.set(Color::Black, pos);
            self.hot_field.set(pos);
        }

        for pos in whites {
            self.slices.set(Color::White, pos);
            self.hot_field.set(pos);
        }

        self.player_color = player;

        self.full_update_mut();
        self.hash_key = HashKey::from(&self.slices.horizontal_slices);
    }

    pub fn switch_player_mut(&mut self) {
        self.player_color = !self.player_color;
    }

    fn incremental_update_mut<const M: MoveType>(&mut self, pos: Pos) {
        macro_rules! update_by_slice_each_color {
            ($color:expr,$direction:expr,$slice:expr,$slice_idx:expr) => {
                match (
                    $slice.pattern_bitmap.get::<{ $color }>() != 0,
                    $slice.has_potential_pattern::<{ $color }>()
                ) {
                    (_, true) => {
                        self.patterns.update_with_slice_mut::<{ RuleKind::Renju }, { $color }, { $direction }>($slice);
                    },
                    (true, false) => {
                        self.patterns.clear_with_slice_mut::<{ $color }, { $direction }>($slice);
                    },
                    _ => {}
                }
            };
        }

        macro_rules! update_by_slice {
            ($direction:expr,$slice:expr,$slice_idx:expr) => {{
                match M {
                    MoveType::Set => $slice.set_mut(self.player_color, $slice_idx),
                    MoveType::Unset => $slice.unset_mut(self.player_color, $slice_idx)
                }

                update_by_slice_each_color!(Color::Black, $direction, $slice, $slice_idx);
                update_by_slice_each_color!(Color::White, $direction, $slice, $slice_idx);
            }};
        }

        self.patterns.unchecked_five_pos = Patterns::EMPTY_UNCHECKED_FIVE_POS;

        let horizontal_slice = &mut self.slices.horizontal_slices[pos.row_usize()];
        update_by_slice!(Direction::Horizontal, horizontal_slice, pos.col());

        let vertical_slice = &mut self.slices.vertical_slices[pos.col_usize()];
        update_by_slice!(Direction::Vertical, vertical_slice, pos.row());

        if let Some(ascending_slice) = self.slices.ascending_slice_mut(pos) {
            update_by_slice!(Direction::Ascending, ascending_slice, pos.col() - ascending_slice.start_col);
        }

        if let Some(descending_slice) = self.slices.descending_slice_mut(pos) {
            update_by_slice!(Direction::Descending, descending_slice, pos.col() - descending_slice.start_col);
        }

        self.validate_forbidden_moves_mut();
    }

    fn full_update_mut(&mut self) {
        macro_rules! update_by_slice {
            ($slice:expr,$direction:expr) => {{
                if $slice.has_potential_pattern::<{ Color::Black }>() {
                    self.patterns.update_with_slice_mut::<{ RuleKind::Renju }, { Color::Black }, { $direction }>($slice);
                }

                if $slice.has_potential_pattern::<{ Color::White }>() {
                    self.patterns.update_with_slice_mut::<{ RuleKind::Renju }, { Color::White }, { $direction }>($slice);
                }
            }};
        }

        self.patterns.unchecked_five_pos = Patterns::EMPTY_UNCHECKED_FIVE_POS;

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

        self.validate_forbidden_moves_mut();
    }

    fn validate_forbidden_moves_mut(&mut self) {
        for root_pos in self.patterns.candidate_forbidden_field.clone().iter_hot_pos() {
            let pattern = self.patterns.field[Color::Black][root_pos.idx_usize()];

            let mark_forbidden: bool;
            let delete_forbidden: bool;

            if pattern.has_five() {
                mark_forbidden = false;
                delete_forbidden = false;
            } else if pattern.has_fours() || pattern.has_overline() {
                mark_forbidden = true;
                delete_forbidden = false;
            } else if pattern.has_threes() {
                if self.is_valid_double_three(ValidateThreeRoot { root_pos }) {
                    mark_forbidden = true;
                    delete_forbidden = false;
                } else {
                    mark_forbidden = false;
                    delete_forbidden = false;
                }
            } else {
                mark_forbidden = false;
                delete_forbidden = true;
            }

            if mark_forbidden {
                self.patterns.forbidden_field.set(root_pos);
            } else {
                self.patterns.forbidden_field.unset(root_pos);
            }

            if delete_forbidden {
                self.patterns.candidate_forbidden_field.unset(root_pos);
            }
        }
    }

    fn is_invalid_three_component<C: ValidateThreeContext>(&self, context: C, direction: Direction, offset: isize) -> bool {
        const ANY_FOUR_OR_OVERLINE_MASK: u32 = pattern::UNIT_ANY_FOUR_MASK | pattern::UNIT_OVERLINE_MASK;

        let pos = context.parent_pos().directional_offset_unchecked(direction, offset);

        let pattern = self.patterns.field[Color::Black][pos.idx_usize()];

        !pattern.has_three() // non-three
            || pattern.apply_mask(ANY_FOUR_OR_OVERLINE_MASK) != 0 // double-four or overline
            || context.override_contains(pos) // double-four or recursive
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
        let pattern_unit = self.patterns.field[Color::Black][pos.idx_usize()];

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
        for direction in self.patterns.field[Color::Black][overrides.root.idx_usize()].iter_three_directions() {
            self.update_four_overrides_each_direction(overrides, direction, overrides.root);
        }
    }

    fn update_four_overrides(&self, overrides: &mut SetOverrides, direction_from: Direction, pos: Pos) {
        for next_four_idx in
            (0 .. direction_from as usize * 3)
                .chain((direction_from as usize + 1) * 3 .. 12)
        {
            let four_pos = overrides.next_four[next_four_idx];
            if four_pos.is_some() {
                overrides.bitfield.set(four_pos.unwrap());
            }
        }

        overrides.next_four = [MaybePos::NONE; 12];

        for direction in self.patterns.field[Color::Black][pos.idx_usize()].iter_three_directions() {
            if direction == direction_from {
                continue;
            }

            self.update_four_overrides_each_direction(overrides, direction, pos);
        }

        overrides.bitfield.set(pos);
    }

    fn update_four_overrides_each_direction(&self, overrides: &mut SetOverrides, direction: Direction, pos: Pos) {
        let direction_offset = direction as usize * 3;

        match self.calculate_near_four_window::<{ Color::Black }>(direction, pos) {
            /* .VOO.  */ 0b11000 => {
                overrides.next_four[direction_offset] = pos.directional_offset_unchecked(direction, -1).into();
                overrides.next_four[direction_offset + 1] = pos.directional_offset_unchecked(direction, 3).into();
            },
            /* .OOV.  */ 0b00011 => {
                overrides.next_four[direction_offset] = pos.directional_offset_unchecked(direction, -3).into();
                overrides.next_four[direction_offset + 1] = pos.directional_offset_unchecked(direction, 1).into();
            },
            /* .V.OO. */ 0b10000 => {
                overrides.next_four[direction_offset] = pos.directional_offset_unchecked(direction, -1).into();
                overrides.next_four[direction_offset + 1] = pos.directional_offset_unchecked(direction, 1).into();
                overrides.next_four[direction_offset + 2] = pos.directional_offset_unchecked(direction, 3).into();
            }
            /* .OO.V. */ 0b00001 => {
                overrides.next_four[direction_offset] = pos.directional_offset_unchecked(direction, -3).into();
                overrides.next_four[direction_offset + 1] = pos.directional_offset_unchecked(direction, -1).into();
                overrides.next_four[direction_offset + 2] = pos.directional_offset_unchecked(direction, 1).into();
            }
            /* .VO.O. */ 0b01000 => {
                overrides.next_four[direction_offset] = pos.directional_offset_unchecked(direction, -1).into();
                overrides.next_four[direction_offset + 1] = pos.directional_offset_unchecked(direction, 2).into();
                overrides.next_four[direction_offset + 2] = pos.directional_offset_unchecked(direction, 4).into();
            },
            /* .OVO.  */ 0b01010 => {
                overrides.next_four[direction_offset] = pos.directional_offset_unchecked(direction, -2).into();
                overrides.next_four[direction_offset + 1] = pos.directional_offset_unchecked(direction, 2).into();
            },
            /* .O.OV. */ 0b00010 => {
                overrides.next_four[direction_offset] = pos.directional_offset_unchecked(direction, -4).into();
                overrides.next_four[direction_offset + 1] = pos.directional_offset_unchecked(direction, -2).into();
                overrides.next_four[direction_offset + 2] = pos.directional_offset_unchecked(direction, 1).into();
            },
            /* .OV.O. */ 0b10010 => {
                overrides.next_four[direction_offset] = pos.directional_offset_unchecked(direction, -2).into();
                overrides.next_four[direction_offset + 1] = pos.directional_offset_unchecked(direction, 1).into();
                overrides.next_four[direction_offset + 2] = pos.directional_offset_unchecked(direction, 3).into();
            },
            /* .O.VO. */ 0b01001 => {
                overrides.next_four[direction_offset] = pos.directional_offset_unchecked(direction, -3).into();
                overrides.next_four[direction_offset + 1] = pos.directional_offset_unchecked(direction, -1).into();
                overrides.next_four[direction_offset + 2] = pos.directional_offset_unchecked(direction, 2).into();
            },
            _ => unreachable!()
        }
    }

    fn calculate_near_four_window<const C: Color>(&self, direction: Direction, pos: Pos) -> u8 {
        let slice = self.slices.access_slice_unchecked(direction, pos);
        let slice_idx = slice.calculate_slice_idx(direction, pos);

        let stones = match C {
            Color::Black => slice.stones[Color::Black],
            Color::White => slice.stones[Color::White]
        } as u32;

        (((stones << 2) >> slice_idx) & 0b11111) as u8 // 0[00V00]0
    }

    pub fn find_winner(&self, pos: Pos) -> Option<Color> {
        [
            Some(&self.slices.horizontal_slices[pos.row_usize()]),
            Some(&self.slices.vertical_slices[pos.col_usize()]),
            self.slices.ascending_slice(pos),
            self.slices.descending_slice(pos),
        ].iter()
            .find_map(|maybe_slice| maybe_slice
                .and_then(Slice::winner)
            )
    }

    pub fn find_global_winner(&self) -> Option<Color> {
        self.slices.horizontal_slices.iter()
            .chain(self.slices.vertical_slices.iter())
            .chain(self.slices.ascending_slices.iter())
            .chain(self.slices.descending_slices.iter())
            .find_map(Slice::winner)
    }

    pub fn is_forced_defense(&self) -> bool {
        self.effective_fours(self.player_color) != 0
    }

    fn effective_fours(&self, color: Color) -> u32 {
        match color {
            Color::White => {
                let mut total_fours = self.patterns.counts.global[Color::Black].open_fours as u32;

                total_fours -= self.patterns.forbidden_field.iter_hot_idx()
                    .map(|idx| self.patterns.field[Color::Black][idx].count_open_fours())
                    .sum::<u32>();

                total_fours
            },
            Color::Black => self.patterns.counts.global[Color::White].open_fours as u32
        }
    }

}

#[derive(std::marker::ConstParamTy, Eq, PartialEq,)]
pub enum MoveType {
    Set, Unset
}

trait ValidateThreeContext : Copy {

    const IS_ROOT: bool;

    fn parent_pos(&self) -> Pos;

    fn parent_direction(&self) -> Direction;

    fn branch_overrides(&self) -> SetOverrides;

    fn override_contains(&self, pos: Pos) -> bool;

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

    fn override_contains(&self, _pos: Pos) -> bool {
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

    fn override_contains(&self, pos: Pos) -> bool {
        self.overrides.bitfield.is_hot(pos)
    }
}

#[derive(Copy, Clone)]
pub struct SetOverrides {
    bitfield: Bitfield,
    next_four: [MaybePos; 12],
    root: Pos,
}

impl SetOverrides {

    fn new(root: Pos) -> Self {
        let mut bitfield = Bitfield::default();

        bitfield.set(root);

        Self {
            bitfield,
            next_four: [MaybePos::NONE; 12],
            root,
        }
    }

}
