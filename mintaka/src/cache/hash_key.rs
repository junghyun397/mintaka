use crate::board::Board;
use crate::cache::hash_table;
use crate::cache::hash_table::EMPTY_HASH;
use crate::notation::color::Color;
use crate::notation::pos::Pos;
use crate::notation::rule;
use crate::slice::Slice;

#[derive(Copy, Clone)]
pub struct HashKey(u64);

impl HashKey {

    pub fn set(self, color: Color, pos: Pos) -> Self {
        self.set_idx(color, pos.idx() as usize)
    }

    pub fn set_idx(self, color: Color, idx: usize) -> Self {
        HashKey(self.0 ^ hash_table::TABLE[match color {
            Color::Black => idx,
            Color::White => rule::BOARD_SIZE as usize + idx,
        }])
    }

}

impl Default for HashKey {

    fn default() -> Self {
        Self(EMPTY_HASH)
    }

}

impl Into<HashKey> for [Slice; rule::U_BOARD_WIDTH] {

    fn into(self) -> HashKey {
        self.iter()
            .enumerate()
            .fold(Default::default(), |mut key, (row, slice)| {
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

impl Into<HashKey> for &Slice {

    fn into(self) -> HashKey {
        HashKey(self.black_stones as u64 | (self.white_stones as u64 >> 16))
    }

}
