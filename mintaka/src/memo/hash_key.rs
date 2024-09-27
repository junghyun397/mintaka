use crate::cartesian_to_index;
use crate::memo::hash_table;
use crate::notation::color::Color;
use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::slice::Slice;

#[derive(Copy, Clone)]
pub struct HashKey(u64);

impl HashKey {

    pub fn set(self, color: Color, pos: Pos) -> Self {
        self.set_idx(color, pos.idx_usize())
    }

    pub fn set_idx(self, color: Color, idx: usize) -> Self {
        HashKey(self.0 ^ match color {
            Color::Black => hash_table::HASH_TABLE[0][idx],
            Color::White => hash_table::HASH_TABLE[1][idx],
        })
    }

}

impl Default for HashKey {

    fn default() -> Self {
        Self(hash_table::EMPTY_HASH)
    }

}

impl From<&[Slice; pos::U_BOARD_WIDTH]> for HashKey {

    fn from(value: &[Slice; pos::U_BOARD_WIDTH]) -> Self {
        value.iter()
            .enumerate()
            .fold(HashKey::default(), |mut key, (row, slice)| {
                for col in 0 .. pos::BOARD_WIDTH {
                    if let Some(color) = slice.stone_kind(col) {
                        key = key.set_idx(color, cartesian_to_index!(row, col as usize))
                    }
                }

                key
            })
    }
}
