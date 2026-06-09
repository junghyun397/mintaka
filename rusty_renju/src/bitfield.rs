use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::{assert_struct_sizes, impl_debug_from_display};
#[cfg(feature = "serde")]
use base64::engine::{general_purpose, Engine as _};
use std::fmt::{Display, Formatter};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};
use std::simd::Simd;
#[cfg(feature = "typeshare")]
use typeshare::typeshare;
use crate::utils::empty::Empty;

// serialization format: base64url-safe no padding
#[cfg_attr(feature = "typeshare", typeshare(serialized_as = "String"))]
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(align(32))]
pub struct Bitfield(pub [u8; 32]);

assert_struct_sizes!(Bitfield, size=32, align=32);

impl Empty for Bitfield {
    fn empty() -> Self {
        Self::ZERO_FILLED
    }
}

impl Bitfield {
    pub const ZERO_FILLED: Bitfield = Bitfield([0; 32]);

    pub const ONE_FILLED: Bitfield = Bitfield([0xFF; 32]);

    pub const fn is_hot_idx(&self, idx: usize) -> bool {
        self.0[idx / 8] & (0b1 << (idx % 8)) != 0
    }

    pub const fn is_hot(&self, pos: Pos) -> bool {
        self.is_hot_idx(pos.idx_usize())
    }

    pub const fn is_cold_idx(&self, idx: usize) -> bool {
        !self.is_hot_idx(idx)
    }

    pub const fn is_cold(&self, pos: Pos) -> bool {
        !self.is_hot_idx(pos.idx_usize())
    }

    pub const fn or_bit_idx(&mut self, idx: usize, bit: bool) {
        self.0[idx / 8] |= (bit as u8) << (idx % 8);
    }

    pub const fn set_bit_idx(&mut self, idx: usize, bit: bool) {
        if bit {
            self.set_idx(idx);
        } else {
            self.unset_idx(idx);
        }
    }

    pub const fn set_idx(&mut self, idx: usize) {
        self.or_bit_idx(idx, true)
    }

    pub const fn set(&mut self, pos: Pos) {
        self.set_idx(pos.idx_usize());
    }

    pub const fn unset_idx(&mut self, idx: usize) {
        self.0[idx / 8] &= !(0b1 << (idx % 8));
    }

    pub const fn unset(&mut self, pos: Pos) {
        self.unset_idx(pos.idx_usize());
    }

    pub fn count_hots(&self) -> u32 {
        self.to_chunks()
            .iter()
            .map(|x| x.count_ones())
            .sum()
    }

    pub fn count_colds(&self) -> u32 {
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
        BitfieldHotBitsIterator::from(self.to_chunks())
    }

    pub fn iter_hot_pos(&self) -> impl Iterator<Item=Pos> + '_ {
        self.iter_hot_idx()
            .map(|x| Pos::from_index(x as u8))
    }

    pub fn first_pos(&self) -> Option<Pos> {
        let chunks = self.to_chunks();

        if chunks[0] != 0 {
            Some(Pos::from_index(chunks[0].trailing_zeros() as u8))
        } else if chunks[1] != 0 {
            Some(Pos::from_index(chunks[1].trailing_zeros() as u8 + 64))
        } else if chunks[2] != 0 {
            Some(Pos::from_index(chunks[2].trailing_zeros() as u8 + 128))
        } else if chunks[3] != 0 {
            Some(Pos::from_index(chunks[3].trailing_zeros() as u8 + 192))
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0 == [0; 32]
    }

    fn to_simd(self) -> Simd<u8, 32> {
        Simd::<u8, 32>::from_array(self.0)
    }

    fn to_chunks(self) -> [u64; 4] {
        [
            u64::from_le_bytes(self.0[0..8].try_into().unwrap()),
            u64::from_le_bytes(self.0[8..16].try_into().unwrap()),
            u64::from_le_bytes(self.0[16..24].try_into().unwrap()),
            u64::from_le_bytes(self.0[24..32].try_into().unwrap()),
        ]
    }
}

impl Not for Bitfield {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self((!self.to_simd()).into())
    }
}

impl BitAnd for Bitfield {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self((self.to_simd() & rhs.to_simd()).into())
    }
}

impl BitOr for Bitfield {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self((self.to_simd() | rhs.to_simd()).into())
    }
}

impl BitXor for Bitfield {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self((self.to_simd() ^ rhs.to_simd()).into())
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

struct BitfieldHotBitsIterator {
    chunks: [u64; 4],
    chunk_mask: usize,
}

impl From<[u64; 4]> for BitfieldHotBitsIterator {
    fn from(chunks: [u64; 4]) -> Self {
        let chunk_mask = (chunks[0] != 0) as usize
            | (((chunks[1] != 0) as usize) << 1)
            | (((chunks[2] != 0) as usize) << 2)
            | (((chunks[3] != 0) as usize) << 3);

        Self {
            chunks,
            chunk_mask,
        }
    }
}

impl ExactSizeIterator for BitfieldHotBitsIterator {
    fn len(&self) -> usize {
        self.chunks.iter()
            .map(|x| x.count_ones() as usize)
            .sum()
    }
}

impl Iterator for BitfieldHotBitsIterator {
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

#[cfg(feature = "serde")]
impl serde::Serialize for Bitfield {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        if serializer.is_human_readable() {
            serializer.serialize_str(&general_purpose::URL_SAFE_NO_PAD.encode(&self.0))
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Bitfield {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let vec = if deserializer.is_human_readable() {
            general_purpose::URL_SAFE_NO_PAD.decode(&String::deserialize(deserializer)?)
                .map_err(serde::de::Error::custom)?
        } else {
            Vec::<u8>::deserialize(deserializer)
                .map_err(serde::de::Error::custom)?
        };

        vec
            .try_into()
            .map_err(|_| serde::de::Error::custom("invalid bitfield binary"))
            .map(Self)
    }
}
