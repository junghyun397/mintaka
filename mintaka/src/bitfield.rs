use crate::notation::pos;
use crate::notation::pos::Pos;
use ethnum::{u256, uint, U256};
use std::ops::{BitAndAssign, BitOrAssign};

pub type Bitfield = U256;

pub trait BitfieldOps {

    fn is_cold(&self, pos: Pos) -> bool;

    fn is_hot(&self, pos: Pos) -> bool;

    fn set(&mut self, pos: Pos);

    fn unset(&mut self, pos: Pos);

    fn iter(&self) -> impl Iterator<Item=bool> + '_;

    fn iter_hot_pos(&self) -> impl Iterator<Item=Pos> + '_;

    fn iter_cold_pos(&self) -> impl Iterator<Item=Pos> + '_;

}

impl BitfieldOps for Bitfield {

    fn is_cold(&self, pos: Pos) -> bool {
        self & (uint!("0b1") << pos.idx()) == 0
    }

    fn is_hot(&self, pos: Pos) -> bool {
        self & (uint!("0b1") << pos.idx()) != 0
    }

    fn set(&mut self, pos: Pos) {
        self.bitor_assign(uint!("0b1") << pos.idx())
    }

    fn unset(&mut self, pos: Pos) {
        self.bitand_assign(!(uint!("0b1") << pos.idx()))
    }

    fn iter(&self) -> impl Iterator<Item=bool> + '_ {
        BitfieldIterator::from(*self)
    }

    fn iter_hot_pos(&self) -> impl Iterator<Item=Pos> + '_ {
        BitfieldPosIterator::from(*self)
    }

    fn iter_cold_pos(&self) -> impl Iterator<Item=Pos> + '_ {
        BitfieldPosIterator::from(!*self)
    }

}

struct BitfieldIterator {
    value: u256,
    position: u8,
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
        if self.position == pos::U8_BOARD_SIZE {
            None
        } else {
            let result = self.value & uint!("0b1") == 1;
            self.position += 1;
            self.value >>= 1;
            Some(result)
        }
    }

}

struct BitfieldPosIterator {
    value: u256
}

impl From<u256> for BitfieldPosIterator {

    fn from(value: u256) -> Self {
        Self { value }
    }

}

impl Iterator for BitfieldPosIterator {

    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        if self.value != u256::MIN {
            let tails = self.value.trailing_zeros();
            self.value &= self.value - 1;
            Some(Pos::from_index(tails as u8))
        } else {
            None
        }
    }

}

impl ExactSizeIterator for BitfieldPosIterator {

    fn len(&self) -> usize {
        self.value.count_ones() as usize
    }

}
