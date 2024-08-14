use crate::board::Board;
use crate::cache::hash_table;
use crate::cache::hash_table::EMPTY_HASH;
use crate::notation::color::Color;
use crate::notation::pos::Pos;
use crate::notation::rule;
use crate::slice::Slice;

pub struct HashKey(u64);

impl HashKey {

    pub fn empty() -> Self {
        HashKey(EMPTY_HASH)
    }

    pub fn set(self, color: Color, pos: Pos) -> Self {
        self.set_idx(color, pos.to_index() as usize)
    }

    fn set_idx(self, color: Color, idx: usize) -> Self {
        HashKey(self.0 ^ hash_table::TABLE[match color {
            Color::Black => idx,
            Color::White => rule::BOARD_SIZE as usize + idx,
        }])
    }

}

impl Into<HashKey> for Board {

    fn into(self) -> HashKey {
        self.slices.vertical_slices
            .iter()
            .enumerate()
            .fold(HashKey::empty(), |mut key, (row, slice)| {
                for col in 0 .. rule::BOARD_WIDTH {
                    if slice.black_stone_at(col) {
                        key = key.set(Color::Black, Pos::from_cartesian(row as u8, col))
                    } else if slice.white_stone_at(col) {
                        key = key.set(Color::White, Pos::from_cartesian(row as u8, col))
                    }
                }

                key
            })
    }

}

impl Into<HashKey> for Slice {

    fn into(self) -> HashKey {
        let mut key = HashKey::empty();

        for idx in 0 .. self.length {
            if self.black_stone_at(idx) {
                key = key.set_idx(Color::Black, idx as usize)
            } else if self.white_stone_at(idx) {
                key = key.set_idx(Color::White, idx as usize)
            }
        }

        key
    }

}
