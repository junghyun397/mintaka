use crate::cache::hash_key::HashKey;
use crate::formation::Formations;
use crate::notation::color::Color;
use crate::notation::direction::Direction;
use crate::notation::pos::{cartesian_to_index, Pos};
use crate::slice::{Slice, Slices};

// 1344-Bytes
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

    pub fn set(&self, pos: Pos) -> Self {
        let mut board = self.clone();
        board.set_mut(pos);
        board
    }

    pub fn unset(&self, pos: Pos) -> Self {
        let mut board = self.clone();
        board.unset_mut(pos);
        board
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

    fn incremental_update_mut(&mut self, pos: Pos, slice_mut_op: fn(&mut Slice, Color, u8)) {
        let horizontal_slice = &mut self.slices.horizontal_slices[pos.row() as usize];
        slice_mut_op(horizontal_slice, self.player_color, pos.row());
        self.formations.update_with_slice_mut(horizontal_slice, Direction::Horizontal, |offset, start_pos|
            cartesian_to_index(start_pos.row(), start_pos.col() + offset as u8) as usize
        );

        let vertical_slice = &mut self.slices.vertical_slices[pos.col() as usize];
        slice_mut_op(vertical_slice, self.player_color, pos.col());
        self.formations.update_with_slice_mut(vertical_slice, Direction::Vertical, |offset, start_pos|
            cartesian_to_index(start_pos.row() + offset as u8, start_pos.col()) as usize
        );

        if let Some(ascending_slice) = self.slices.access_ascending_slice(pos) {
            slice_mut_op(ascending_slice, self.player_color, pos.col() - ascending_slice.start_pos.col());
            self.formations.update_with_slice_mut(ascending_slice, Direction::Ascending, |offset, start_pos|
                cartesian_to_index(start_pos.row() + offset as u8, start_pos.col() + offset as u8) as usize
            )
        }

        if let Some(descending_slice) = self.slices.access_descending_slice(pos) {
            slice_mut_op(descending_slice, self.player_color, pos.col() - descending_slice.start_pos.col());
            self.formations.update_with_slice_mut(descending_slice, Direction::Descending, |offset, start_pos|
                cartesian_to_index(start_pos.row() - offset as u8, start_pos.col() + offset as u8) as usize
            )
        }
    }

    fn full_update_mut(&mut self) {
        for horizontal_slice in self.slices.horizontal_slices.iter() {
            self.formations.update_with_slice_mut(horizontal_slice, Direction::Horizontal, |offset, start_pos|
                cartesian_to_index(start_pos.row(), start_pos.col() + offset as u8) as usize
            );
        }

        for vertical_slice in self.slices.vertical_slices.iter() {
            self.formations.update_with_slice_mut(vertical_slice, Direction::Vertical, |offset, start_pos|
                cartesian_to_index(start_pos.row() + offset as u8, start_pos.col()) as usize
            );
        }

        for ascending_slice in self.slices.ascending_slices.iter() {
            self.formations.update_with_slice_mut(ascending_slice, Direction::Ascending, |offset, start_pos|
                cartesian_to_index(start_pos.row() + offset as u8, start_pos.col() + offset as u8) as usize
            )
        }

        for descending_slice in self.slices.descending_slices.iter() {
            self.formations.update_with_slice_mut(descending_slice, Direction::Descending, |offset, start_pos|
                cartesian_to_index(start_pos.row() - offset as u8, start_pos.col() + offset as u8) as usize
            )
        }
    }

    fn switch_player_mut(&mut self) {
        self.player_color = self.opponent_color();
    }

}
