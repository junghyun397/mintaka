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
    black_stones: u16,
    white_stones: u16
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
        let mask = 0b1000_0000_0000_0000 >> idx;
        self.black_stones & mask == mask
    }

    pub fn white_stone_at(&self, idx: u8) -> bool {
        let mask = 0b1000_0000_0000_0000 >> idx;
        self.white_stones & mask == mask
    }

    pub fn stone_at(&self, color: &Color, idx: u8) -> bool {
        match color {
            Color::Black => self.black_stone_at(idx),
            Color::White => self.white_stone_at(idx)
        }
    }

    pub fn stone_kind(&self, idx: u8) -> Option<Color> {
        if self.black_stone_at(idx) {
            Some(Color::Black)
        } else if self.white_stone_at(idx) {
            Some(Color::White)
        } else {
            None
        }
    }

    pub fn slice_key(&self) -> SliceKey {
        self.black_stones as u32 | self.white_stones as u32 >> rule::BOARD_WIDTH
    }

}

#[derive(Copy, Clone)]
pub struct Slices {
    pub horizontal_slices: [Slice; rule::U_BOARD_WIDTH],
    pub vertical_slices: [Slice; rule::U_BOARD_WIDTH],
    pub ascending_slices: [Slice; DIAGONAL_SLICE_AMOUNT],
    pub descending_slices: [Slice; DIAGONAL_SLICE_AMOUNT],
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
