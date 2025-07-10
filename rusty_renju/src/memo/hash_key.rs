use crate::cartesian_to_index;
use crate::memo::hash_table;
use crate::notation::color::Color;
use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::slice::Slice;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Copy, Clone, Debug)]
pub struct HashKey(pub u64);

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

impl Serialize for HashKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        if serializer.is_human_readable() {
            format!("{:016x}", self.0).serialize(serializer)
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for HashKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        if deserializer.is_human_readable() {
            u64::from_str_radix(String::deserialize(deserializer)?.as_str(), 16)
                .map(Self)
                .map_err(serde::de::Error::custom)
        } else {
            Ok(Self(u64::deserialize(deserializer)?))
        }
    }
}
