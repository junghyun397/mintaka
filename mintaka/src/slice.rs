use crate::bitfield::Bitfield;
use crate::notation::color::Color;
use crate::notation::direction::Direction;
use crate::notation::pos;
use crate::notation::pos::Pos;
use ethnum::{u256, U256};
use std::cmp::max;
use std::ops::Neg;

const DIAGONAL_SLICE_AMOUNT: usize = pos::U_BOARD_WIDTH * 2 - 4 - 4 - 1;
const I_DIAGONAL_SLICE_AMOUNT: isize = DIAGONAL_SLICE_AMOUNT as isize;

#[derive(Debug, Copy, Clone)]
pub struct Slice {
    pub length: u8,
    pub start_pos: Pos,
    pub black_stones: u16,
    pub white_stones: u16
}

pub type SliceKey = u32;

impl Slice {

    pub fn empty(length: u8, start_pos: Pos) -> Self {
        Slice {
            length,
            start_pos,
            black_stones: 0,
            white_stones: 0
        }
    }

    pub fn set(mut self, color: Color, idx: u8) -> Self {
        self.set_mut(color, idx);
        self
    }

    pub fn unset(mut self, color: Color, idx: u8) -> Self {
        self.unset_mut(color, idx);
        self
    }

    pub fn set_mut(&mut self, color: Color, idx: u8) {
        let mask = 0b1 << idx;
        match color {
            Color::Black => self.black_stones |= mask,
            Color::White => self.white_stones |= mask
        }
    }

    pub fn unset_mut(&mut self, color: Color, idx: u8) {
        let mask = !(0b1 << idx);
        match color {
            Color::Black => self.black_stones &= mask,
            Color::White => self.white_stones &= mask
        }
    }

    pub fn is_no_joy(&self) -> bool {
        self.black_stones.count_ones() < 2 && self.white_stones.count_ones() < 2
    }

    pub fn stone_kind(&self, idx: u8) -> Option<Color> {
        let mask = 0b1 << idx;

        if self.black_stones & mask == mask {
            Some(Color::Black)
        } else if self.white_stones & mask == mask {
            Some(Color::White)
        } else {
            None
        }
    }

    pub fn slice_key(&self) -> SliceKey {
        self.black_stones as u32 | (self.white_stones as u32) << pos::BOARD_WIDTH
    }

}

#[derive(Copy, Clone)]
pub struct Slices {
    pub horizontal_slices: [Slice; pos::U_BOARD_WIDTH],
    pub vertical_slices: [Slice; pos::U_BOARD_WIDTH],
    pub ascending_slices: [Slice; DIAGONAL_SLICE_AMOUNT],
    pub descending_slices: [Slice; DIAGONAL_SLICE_AMOUNT],
}

impl Default for Slices {

    fn default() -> Self {
        Self {
            horizontal_slices: std::array::from_fn(|idx|
                Slice::empty(pos::BOARD_WIDTH, Pos::from_cartesian(idx as u8, 0))
            ),
            vertical_slices: std::array::from_fn(|idx|
                Slice::empty(pos::BOARD_WIDTH, Pos::from_cartesian(0, idx as u8))
            ),
            ascending_slices: std::array::from_fn(|idx| {
                let seq_num = idx as isize + 5 - pos::I_BOARD_WIDTH;
                Slice::empty(
                    (seq_num.abs() - pos::I_BOARD_WIDTH).unsigned_abs() as u8,
                    Pos::from_cartesian(max(0, seq_num.neg()) as u8, max(0, seq_num) as u8)
                )
            }),
            descending_slices: std::array::from_fn(|idx| {
                let seq_num = idx as isize + 5 - pos::I_BOARD_WIDTH;
                Slice::empty(
                    (seq_num.abs() - pos::I_BOARD_WIDTH).unsigned_abs() as u8,
                    Pos::from_cartesian(
                        pos::BOARD_WIDTH-1 - max(0, seq_num.neg()) as u8,
                        max(0, seq_num) as u8
                    )
                )
            })
        }
    }

}

impl Slices {

    pub fn set_mut(&mut self, color: Color, pos: Pos) {
        self.horizontal_slices[pos.row_usize()].set_mut(color, pos.col());
        self.vertical_slices[pos.col_usize()].set_mut(color, pos.row());

        if let Some(ascending_slice) = self.occupy_ascending_slice(pos) {
            ascending_slice.set_mut(color, pos.col() - ascending_slice.start_pos.col())
        }

        if let Some(descending_slice) = self.occupy_descending_slice(pos) {
            descending_slice.set_mut(color, pos.col() - descending_slice.start_pos.col())
        }
    }

    pub fn access_slice<const D: Direction>(&self, pos: Pos) -> Option<&Slice> {
        match D {
            Direction::Horizontal => Some(&self.horizontal_slices[pos.row_usize()]),
            Direction::Vertical => Some(&self.vertical_slices[pos.col_usize()]),
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

    pub fn occupy_ascending_slice(&mut self, pos: Pos) -> Option<&mut Slice> {
        Self::ascending_slice_idx(pos)
            .map(|idx| &mut self.ascending_slices[idx])
    }

    pub fn occupy_descending_slice(&mut self, pos: Pos) -> Option<&mut Slice> {
        Self::descending_slice_idx(pos)
            .map(|idx| &mut self.descending_slices[idx])
    }

    fn ascending_slice_idx(pos: Pos) -> Option<usize> {
        // y = x + b, b = y - x (reversed row sequence)
        let idx = I_DIAGONAL_SLICE_AMOUNT - (pos.row() as isize - pos.col() as isize + pos::I_BOARD_WIDTH - 4);
        (0 .. I_DIAGONAL_SLICE_AMOUNT).contains(&idx)
            .then_some(idx as usize)
    }

    fn descending_slice_idx(pos: Pos) -> Option<usize> {
        // y = -x + 15 + b, b = y + x - 15
        let idx = pos.row() as isize + pos.col() as isize - 4;
        (0 .. I_DIAGONAL_SLICE_AMOUNT).contains(&idx)
            .then_some(idx as usize)
    }

    pub fn non_empties(&self) -> Bitfield {
        self.horizontal_slices.iter()
            .enumerate()
            .fold(u256::MIN, |mut acc, (row_idx, row)| {
                acc |= U256::from((row.black_stones | row.white_stones) << (row_idx * pos::U_BOARD_WIDTH));
                acc
            })
    }

    pub fn empties(&self) -> Bitfield {
        !self.non_empties()
    }

}
