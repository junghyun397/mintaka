use crate::assert_struct_sizes;
use crate::notation::pos;
use crate::notation::pos::Pos;
use ethnum::u256;
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
        self.to_u256().count_ones()
    }

    pub fn count_zeros(&self) -> u32 {
        pos::BOARD_SIZE as u32 - self.count_ones()
    }

    pub fn iter(&self) -> impl Iterator<Item=bool> + '_ {
        BitfieldIterator::from(self.to_u256())
    }

    pub fn iter_hot_idx(&self) -> impl Iterator<Item=usize> + '_ {
        BitfieldSetBitsIterator::from(self.to_u256())
            .map(|x| x as usize)
    }

    pub fn iter_hot_pos(&self) -> impl Iterator<Item=Pos> + '_ {
        BitfieldSetBitsIterator::from(self.to_u256())
            .map(|x| Pos::from_index(x as u8))
    }

    pub fn iter_cold_pos(&self) -> impl Iterator<Item=Pos> + '_ {
        BitfieldSetBitsIterator::from(!self.to_u256())
            .map(|x| Pos::from_index(x as u8))
    }

    pub fn is_empty(&self) -> bool {
        self.to_u256() == u256::MIN
    }

    pub fn to_simd(self) -> u8x32 {
        u8x32::from_array(self.0)
    }

    pub fn to_u256(self) -> u256 {
        u256::from_ne_bytes(self.0)
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

impl From<u8x32> for Bitfield {

    fn from(x: u8x32) -> Self {
        Self(x.to_array())
    }

}

impl From<u256> for Bitfield {

    fn from(x: u256) -> Self {
        Self(x.to_ne_bytes())
    }

}

struct BitfieldIterator {
    value: u256,
    position: usize,
}

impl From<u256> for BitfieldIterator {

    fn from(value: u256) -> Self {
        Self {
            value,
            position: 0,
        }
    }

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
            None
        } else {
            let result = self.value.as_u8() & 0b1 == 1;
            self.position += 1;
            self.value >>= 1;
            Some(result)
        }
    }

}

struct BitfieldSetBitsIterator {
    value: u256
}

impl From<u256> for BitfieldSetBitsIterator {

    fn from(value: u256) -> Self {
        Self { value }
    }

}

impl Iterator for BitfieldSetBitsIterator {

    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.value != u256::MIN {
            let idx = self.value.trailing_zeros();
            self.value &= self.value - 1;
            Some(idx)
        } else {
            None
        }
    }

}

impl ExactSizeIterator for BitfieldSetBitsIterator {

    fn len(&self) -> usize {
        self.value.count_ones() as usize
    }

}
