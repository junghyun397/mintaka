use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::{assert_struct_sizes, impl_debug_from_display};
use base64::prelude::BASE64_URL_SAFE;
use base64::Engine;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};
use std::simd::u8x32;

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(align(32))]
pub struct Bitfield(pub [u8; 32]);

assert_struct_sizes!(Bitfield, size=32, align=32);

impl Default for Bitfield {

    fn default() -> Self {
        Self::ZERO_FILLED
    }

}

impl Bitfield {

    pub const ZERO_FILLED: Bitfield = Bitfield([0; 32]);

    pub const fn is_cold(&self, pos: Pos) -> bool {
        self.0[pos.idx_usize() / 8] & (0b1 << (pos.idx_usize() % 8)) == 0
    }

    pub const fn is_hot(&self, pos: Pos) -> bool {
        self.0[pos.idx_usize() / 8] & (0b1 << (pos.idx_usize() % 8)) != 0
    }

    pub const fn set_mut(&mut self, pos: Pos) {
        self.0[pos.idx_usize() / 8] |= 0b1 << (pos.idx() % 8);
    }

    pub const fn unset_mut(&mut self, pos: Pos) {
        self.0[pos.idx_usize() / 8] &= !(0b1 << (pos.idx() % 8));
    }

    pub fn count_ones(&self) -> u32 {
        self.to_chunks()
            .iter()
            .map(|x| x.count_ones())
            .sum()
    }

    pub fn count_zeros(&self) -> u32 {
        self.to_chunks()
            .iter()
            .map(|x| x.count_zeros())
            .sum()
    }

    pub fn iter(&self) -> impl Iterator<Item=bool> + '_ {
        BitfieldIterator {
            chunks: self.to_chunks(),
            position: 0,
        }
    }

    pub fn iter_hot_idx(&self) -> impl Iterator<Item=usize> + '_ {
        BitfieldSetBitsIterator::from(self.to_chunks())
    }

    pub fn iter_hot_pos(&self) -> impl Iterator<Item=Pos> + '_ {
        self.iter_hot_idx()
            .map(|x| Pos::from_index(x as u8))
    }

    pub fn iter_cold_pos(&self) -> impl Iterator<Item=Pos> + '_ {
        BitfieldSetBitsIterator::from((!*self).to_chunks())
            .map(|x| Pos::from_index(x as u8))
    }

    pub fn is_empty(&self) -> bool {
        self.0 == [0; 32]
    }

    pub fn to_simd(self) -> u8x32 {
        u8x32::from_array(self.0)
    }

    pub fn to_chunks(self) -> [u64; 4] {
        unsafe { std::mem::transmute::<[u8; 32], [u64; 4]>(self.0) }
    }

}

impl Not for Bitfield {
    type Output = Self;

    fn not(self) -> Self::Output {
        (!self.to_simd()).into()
    }
}

impl BitAnd for Bitfield {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        (self.to_simd() & rhs.to_simd()).into()
    }
}

impl BitOr for Bitfield {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        (self.to_simd() | rhs.to_simd()).into()
    }
}

impl BitXor for Bitfield {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        (self.to_simd() ^ rhs.to_simd()).into()
    }
}

impl BitOrAssign for Bitfield {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.bitor(rhs);
    }
}

impl BitAndAssign for Bitfield {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.bitand(rhs);
    }
}

impl BitXorAssign for Bitfield {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = self.bitxor(rhs);
    }
}

impl TryFrom<Vec<u8>> for Bitfield {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into().map_err(|_| "bitfield binary must exactly 32 bytes")?))
    }
}

impl From<u8x32> for Bitfield {

    fn from(x: u8x32) -> Self {
        Self(x.to_array())
    }

}

struct BitfieldIterator {
    chunks: [u64; 4],
    position: usize,
}

impl ExactSizeIterator for BitfieldIterator {

    fn len(&self) -> usize {
        pos::BOARD_SIZE
    }

}

impl Iterator for BitfieldIterator {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position == pos::BOARD_SIZE {
            return None;
        }

        let result = self.chunks[self.position / 64] & 0b1 == 0b1;
        self.chunks[self.position / 64] >>= 1;
        self.position += 1;
        Some(result)
    }

}

struct BitfieldSetBitsIterator {
    chunks: [u64; 4],
    chunk_mask: usize,
}

impl From<[u64; 4]> for BitfieldSetBitsIterator {
    fn from(chunks: [u64; 4]) -> Self {
        let mut chunk_mask = 0;

        chunk_mask |= (chunks[0] != 0) as usize;
        chunk_mask |= ((chunks[1] != 0) as usize) << 1;
        chunk_mask |= ((chunks[2] != 0) as usize) << 2;
        chunk_mask |= ((chunks[3] != 0) as usize) << 3;

        Self {
            chunks,
            chunk_mask,
        }
    }
}

impl ExactSizeIterator for BitfieldSetBitsIterator {
    fn len(&self) -> usize {
        self.chunks.iter()
            .map(|x| x.count_ones() as usize)
            .sum()
    }
}

impl Iterator for BitfieldSetBitsIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.chunk_mask == 0 {
            return None;
        }

        let chunk_idx = self.chunk_mask.trailing_zeros() as usize;
        let idx = chunk_idx * 64 + self.chunks[chunk_idx].trailing_zeros() as usize;

        self.chunks[chunk_idx] &= self.chunks[chunk_idx] - 1;
        self.chunk_mask &= !(((self.chunks[chunk_idx] == 0) as usize) << chunk_idx);

        Some(idx)
    }

}

impl Display for Bitfield {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let content = self.iter()
            .map(|is_hot|
                if is_hot { "X" } else { "." }
            )
            .collect::<Vec<_>>()
            .chunks(pos::U_BOARD_WIDTH)
            .rev()
            .map(|row| row.join(" "))
            .collect::<Vec<_>>()
            .join("\n");

        write!(f, "{content}")
    }
}

impl_debug_from_display!(Bitfield);

impl Serialize for Bitfield {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        if serializer.is_human_readable() {
            BASE64_URL_SAFE.encode(self.0).serialize(serializer)
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

impl<'de> Deserialize<'de> for Bitfield {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        if deserializer.is_human_readable() {
            let binary = BASE64_URL_SAFE.decode(String::deserialize(deserializer)?)
                .map_err(serde::de::Error::custom)?;

            binary.try_into()
                .map_err(serde::de::Error::custom)
        } else {
            Ok(Self(Deserialize::deserialize(deserializer)?))
        }
    }
}
