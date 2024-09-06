use crate::board_width;
use crate::cache::hash_table;
use crate::cache::hash_table::EMPTY_HASH;
use crate::cartesian_to_index;
use crate::notation::color::Color;
use crate::notation::pos::Pos;
use crate::notation::rule;
use crate::notation::rule::U_BOARD_WIDTH;
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
            Color::White => rule::BOARD_SIZE + idx,
        }])
    }

}

impl Default for HashKey {

    fn default() -> Self {
        Self(EMPTY_HASH)
    }

}

impl From<&[Slice; U_BOARD_WIDTH]> for HashKey {

    fn from(value: &[Slice; U_BOARD_WIDTH]) -> Self {
        value.iter()
            .enumerate()
            .fold(HashKey::default(), |mut key, (row, slice)| {
                for col in 0 .. rule::BOARD_WIDTH {
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
