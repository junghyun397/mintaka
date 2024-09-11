use crate::cache::hash_table;
use crate::cartesian_to_index;
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
        HashKey(self.0 ^ hash_table::TABLE[match color {
            Color::Black => idx,
            Color::White => pos::BOARD_SIZE + idx,
        }])
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
                    if slice.black_stone_at(col) {
                        key = key.set_idx(Color::Black, cartesian_to_index!(row, col as usize))
                    } else if slice.white_stone_at(col) {
                        key = key.set_idx(Color::White, cartesian_to_index!(row, col as usize))
                    }
                }

                key
            })
    }
}
