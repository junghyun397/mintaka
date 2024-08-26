use crate::cache::hash_key::HashKey;
use crate::notation::color::Color;
use crate::notation::direction::Direction;
use crate::notation::pos::Pos;
use crate::notation::rule;
use std::cmp::max;
use std::ops::Neg;

const DIAGONAL_SLICE_AMOUNT: usize = rule::U_BOARD_WIDTH * 2 - 4 - 4 - 1;
const SLICE_AMOUNT: usize = rule::U_BOARD_WIDTH * 2 + DIAGONAL_SLICE_AMOUNT * 2;

const I_DIAGONAL_SLICE_AMOUNT: isize = DIAGONAL_SLICE_AMOUNT as isize;

#[derive(Debug, Copy, Clone)]
pub struct Slice {
    pub length: u8,
    pub start_pos: Pos,
    pub black_stones: u16,
    pub white_stones: u16
}

impl Slice {

    pub fn empty(length: u8, start_pos: Pos) -> Self {
        Slice {
            length,
            start_pos,
            black_stones: 0,
            white_stones: 0
        }
    }

    pub fn set(&self, color: Color, idx: u8) -> Self {
        let mut slice = self.clone();
        slice.set_mut(color, idx);

        slice
    }

    pub fn unset(&self, color: Color, idx: u8) -> Self {
        let mut slice = self.clone();
        slice.unset_mut(color, idx);

        slice
    }

    pub fn set_mut(&mut self, color: Color, idx: u8) {
        let mask = 0b1000_0000_0000_0000 >> idx;
        match color {
            Color::Black => self.black_stones = self.black_stones | mask,
            Color::White => self.white_stones = self.white_stones | mask
        }
    }

    pub fn unset_mut(&mut self, color: Color, idx: u8) {
        let mask = !(0b1000_0000_0000_0000 >> idx);
        match color {
            Color::Black => self.black_stones = self.black_stones & mask,
            Color::White => self.white_stones = self.white_stones & mask
        }
    }

    pub fn black_stone_at(&self, idx: u8) -> bool {
        self.black_stones & 0b1000_0000_0000_0000 >> idx == 0b1
    }

    pub fn white_stone_at(&self, idx: u8) -> bool {
        self.white_stones & 0b1000_0000_0000_0000 >> idx == 0b1
    }

    pub fn stone_at(&self, color: &Color, idx: u8) -> bool {
        match color {
            Color::Black => self.black_stone_at(idx),
            Color::White => self.white_stone_at(idx)
        }
    }

    pub fn hash_key(&self) -> HashKey {
        self.into()
    }

}



#[derive(Copy, Clone)]
pub struct Slices {
    pub horizontal_slices: [Slice; rule::U_BOARD_WIDTH],
    pub vertical_slices: [Slice; rule::U_BOARD_WIDTH],
    pub ascending_slices: [Slice; DIAGONAL_SLICE_AMOUNT],
    pub descending_slices: [Slice; DIAGONAL_SLICE_AMOUNT]
}

#[derive(Copy, Clone)]
pub struct SliceSet {
    pub horizontal_slice: Slice,
    pub vertical_slice: Slice,
    pub ascending_slice: Option<Slice>,
    pub descending_slice: Option<Slice>
}

impl Default for Slices {

    fn default() -> Self {
        Self {
            horizontal_slices: std::array::from_fn(|idx|
                Slice::empty(rule::BOARD_WIDTH, Pos::from_cartesian(idx as u8, 0))
            ),
            vertical_slices: std::array::from_fn(|idx|
                Slice::empty(rule::BOARD_WIDTH, Pos::from_cartesian(0, idx as u8))
            ),
            ascending_slices: std::array::from_fn(|idx| {
                let seq_num = idx as isize + 5 - rule::I_BOARD_WIDTH;
                Slice::empty(
                    (seq_num.abs() - rule::I_BOARD_WIDTH).abs() as u8,
                    Pos::from_cartesian(max(0, seq_num.neg()) as u8, max(0, seq_num) as u8)
                )
            }),
            descending_slices: std::array::from_fn(|idx| {
                let seq_num = idx as isize + 5 - rule::I_BOARD_WIDTH;
                Slice::empty(
                    (seq_num.abs() - rule::I_BOARD_WIDTH).abs() as u8,
                    Pos::from_cartesian(
                        rule::BOARD_WIDTH-1 - max(0, seq_num.neg()) as u8,
                        max(0, seq_num) as u8
                    )
                )
            })
        }
    }

}

impl Slices {

    pub fn access_slice(&self, direction: Direction, pos: Pos) -> Option<&Slice> {
        match direction {
            Direction::Horizontal => Some(&self.horizontal_slices[pos.row() as usize]),
            Direction::Vertical => Some(&self.vertical_slices[pos.col() as usize]),
            Direction::Ascending => {
                Self::ascending_slice_idx(pos)
                    .map(|idx| &self.ascending_slices[idx])
            },
            Direction::Descending => {
                Self::descending_slice_idx(pos)
                    .map(|idx| &self.descending_slices[idx])
            }
        }
    }

    pub fn access_ascending_slice(&mut self, pos: Pos) -> Option<&mut Slice> {
        Self::ascending_slice_idx(pos)
            .map(|idx| &mut self.ascending_slices[idx])
    }

    pub fn access_descending_slice(&mut self, pos: Pos) -> Option<&mut Slice> {
        Self::descending_slice_idx(pos)
            .map(|idx| &mut self.descending_slices[idx])
    }

    #[inline(always)]
    pub fn ascending_slice_idx(pos: Pos) -> Option<usize> {
        // y = x + b, b = y - x (reversed row sequence)
        let idx = I_DIAGONAL_SLICE_AMOUNT - (pos.row() as isize - pos.col() as isize + rule::I_BOARD_WIDTH - 4);
        if 0 <= idx && idx < I_DIAGONAL_SLICE_AMOUNT {
            Some(idx as usize)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn descending_slice_idx(pos: Pos) -> Option<usize> {
        // y = -x + 15 + b, b = y + x - 15
        let idx = pos.row() as isize + pos.col() as isize - 4;
        if 0 <= idx && idx < I_DIAGONAL_SLICE_AMOUNT {
            Some(idx as usize)
        } else {
            None
        }
    }

}
