use crate::cache::hash_key::HashKey;
use crate::formation::Formations;
use crate::notation::color::Color;
use crate::notation::direction::Direction;
use crate::notation::pos::Pos;
use crate::slice::{Slice, Slices};

// 2248-bytes
#[derive(Copy, Clone)]
pub struct Board {
    pub player_color: Color,
    pub slices: Slices,
    pub formations: Formations,
    pub hash_key: HashKey,
}

impl Default for Board {

    fn default() -> Self {
        Self {
            player_color: Color::Black,
            slices: Slices::default(),
            formations: Formations::default(),
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
        self.set_mut(pos);
        self
    }

    pub fn unset(mut self, pos: Pos) -> Self {
        self.unset_mut(pos);
        self
    }

    pub fn pass(&self) -> Self {
        let mut board = self.clone();
        board.pass_mut();
        board
    }

    pub fn set_mut(&mut self, pos: Pos) {
        self.incremental_update_mut(pos, Slice::set_mut);

        self.switch_player_mut();
        self.hash_key = self.hash_key.set(self.player_color, pos);
    }

    pub fn unset_mut(&mut self, pos: Pos) {
        self.incremental_update_mut(pos, Slice::unset_mut);

        self.switch_player_mut();
        self.hash_key = self.hash_key.set(self.player_color, pos);
    }

    pub fn pass_mut(&mut self) {
        self.switch_player_mut();
    }

    pub fn batch_set_mut(&mut self, blacks: Vec<Pos>, whites: Vec<Pos>, player: Color) {
        for pos in blacks {
            self.slices.set_mut(Color::Black, pos);
        }

        for pos in whites {
            self.slices.set_mut(Color::White, pos);
        }

        self.player_color = player;
        self.full_update_mut();
    }

    #[inline(always)]
    fn incremental_update_mut(&mut self, pos: Pos, slice_mut_op: fn(&mut Slice, Color, u8)) {
        let horizontal_slice = &mut self.slices.horizontal_slices[pos.row_usize()];
        slice_mut_op(horizontal_slice, self.player_color, pos.col());
        self.formations.update_with_slice_mut::<{ Direction::Horizontal }>(horizontal_slice);

        let vertical_slice = &mut self.slices.vertical_slices[pos.col_usize()];
        slice_mut_op(vertical_slice, self.player_color, pos.row());
        self.formations.update_with_slice_mut::<{ Direction::Vertical }>(vertical_slice);

        if let Some(ascending_slice) = self.slices.occupy_ascending_slice(pos) {
            slice_mut_op(ascending_slice, self.player_color, pos.col() - ascending_slice.start_pos.col());
            self.formations.update_with_slice_mut::<{ Direction::Ascending }>(ascending_slice);
        }

        if let Some(descending_slice) = self.slices.occupy_descending_slice(pos) {
            slice_mut_op(descending_slice, self.player_color, pos.col() - descending_slice.start_pos.col());
            self.formations.update_with_slice_mut::<{ Direction::Descending }>(descending_slice);
        }
    }

    fn full_update_mut(&mut self) {
        for horizontal_slice in self.slices.horizontal_slices.iter() {
            self.formations.update_with_slice_mut::<{ Direction::Horizontal }>(horizontal_slice);
        }

        for vertical_slice in self.slices.vertical_slices.iter() {
            self.formations.update_with_slice_mut::<{ Direction::Vertical }>(vertical_slice);
        }

        for ascending_slice in self.slices.ascending_slices.iter() {
            self.formations.update_with_slice_mut::<{ Direction::Ascending }>(ascending_slice);
        }

        for descending_slice in self.slices.descending_slices.iter() {
            self.formations.update_with_slice_mut::<{ Direction::Descending }>(descending_slice);
        }
    }

    fn switch_player_mut(&mut self) {
        self.player_color = self.opponent_color();
    }

}
