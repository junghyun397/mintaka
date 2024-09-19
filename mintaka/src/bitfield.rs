use crate::notation::pos;
use crate::notation::pos::Pos;
use ethnum::{uint, U256};

pub type Bitfield = U256;

pub fn is_cold(bit_field: Bitfield, pos: Pos) -> bool {
    let mask: U256 = uint!("0b1") << pos.idx();
    bit_field & mask == 0
}

pub fn is_hot(bit_field: Bitfield, pos: Pos) -> bool {
    !is_cold(bit_field, pos)
}

struct BitfieldIterator {
    value: U256,
    position: u8,
}

impl From<U256> for BitfieldIterator {

    fn from(value: U256) -> Self {
        Self {
            value,
            position: 0,
        }
    }

}


impl Iterator for BitfieldIterator {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position == pos::BOARD_SIZE as u8 {
            return None
        }

        let mask = self.value << self.position;

        self.position += 1;
        Some(self.value & mask == mask)
    }

}

struct BitfieldPosIterator {
    value: U256,
    position: u8,
}

impl From<U256> for BitfieldPosIterator {

    fn from(value: U256) -> Self {
        Self {
            value,
            position: 0,
        }
    }

}

impl Iterator for BitfieldPosIterator {

    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position == pos::BOARD_SIZE as u8 {
            return None
        }

        let mask = self.value << self.position;
        let result = (self.value & mask == mask)
            .then(|| Pos::from_index(self.position));

        self.position += 1;
        result
    }

}
