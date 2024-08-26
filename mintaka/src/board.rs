use crate::cache::hash_key::HashKey;
use crate::notation::color::Color;
use crate::formation::Cells;
use crate::notation::pos::Pos;
use crate::notation::rule::RuleKind;
use crate::slice::{Slice, Slices};

// 1344-Bytes
#[derive(Copy, Clone)]
pub struct Board {
    pub player_color: Color,
    pub slices: Slices,
    pub cells: Cells,
    pub hash_key: HashKey,
}

impl Default for Board {

    fn default() -> Self {
        Self {
            player_color: Color::Black,
            slices: Default::default(),
            cells: Default::default(),
            hash_key: Default::default()
        }
    }

}

impl Board {

    pub fn opponent_color(&self) -> Color {
        self.player_color.reversed()
    }

    pub fn set(&self, pos: Pos, rule_kind: RuleKind) -> Self {
        let mut board = self.clone();
        board.set_mut(pos, rule_kind);
        board
    }

    pub fn unset(&self, pos: Pos, rule_kind: RuleKind) -> Self {
        let mut board = self.clone();
        board.unset_mut(pos, rule_kind);
        board
    }

    pub fn pass(&self) -> Self {
        let mut board = self.clone();
        board.pass_mut();
        board
    }

    pub fn set_mut(&mut self, pos: Pos, rule_kind: RuleKind) {
        self.incremental_update_mut(pos, rule_kind, Slice::set_mut);

        self.switch_player_mut();
        self.hash_key = self.hash_key.set(self.player_color, pos);
    }

    pub fn unset_mut(&mut self, pos: Pos, rule_kind: RuleKind) {
        self.incremental_update_mut(pos, rule_kind, Slice::unset_mut);

        self.switch_player_mut();
        self.hash_key = self.hash_key.set(self.player_color, pos);
    }

    pub fn pass_mut(&mut self) {
        self.switch_player_mut();
    }

    pub fn batch_set_mut(&mut self, blacks: Vec<Pos>, whites: Vec<Pos>, next_player: Color, rule_kind: RuleKind) {
        self.player_color = next_player;
        self.full_update_mut(rule_kind)
    }

    fn incremental_update_mut(&mut self, pos: Pos, rule_kind: RuleKind, slice_op: fn(&mut Slice, Color, u8)) {
        let horizontal_slice = &mut self.slices.horizontal_slices[pos.row() as usize];
        slice_op(horizontal_slice, self.player_color, pos.row());

        let vertical_slice = &mut self.slices.vertical_slices[pos.col() as usize];
        slice_op(vertical_slice, self.player_color, pos.col());

        if let Some(ascending_slice) = self.slices.access_ascending_slice(pos) {
            slice_op(ascending_slice, self.player_color, pos.col() - ascending_slice.start_pos.col());
        }

        if let Some(descending_slice) = self.slices.access_descending_slice(pos) {
            slice_op(descending_slice, self.player_color, pos.col() - descending_slice.start_pos.col());
        }

        todo!()
    }

    fn full_update_mut(&mut self, rule_kind: RuleKind) {
        for vertical_slice in self.slices.vertical_slices {

        }

        for horizontal_slice in self.slices.horizontal_slices {

        }

        for ascending_slice in self.slices.ascending_slices {

        }

        for descending_slice in self.slices.descending_slices {

        }

        todo!()
    }

    fn switch_player_mut(&mut self) {
        self.player_color = self.opponent_color();
    }

}
