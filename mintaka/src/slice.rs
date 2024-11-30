use crate::notation::color::Color;
use crate::notation::direction::Direction;
use crate::notation::pos;
use crate::notation::pos::Pos;
use std::cmp::max;
use std::ops::Neg;

const DIAGONAL_SLICE_AMOUNT: usize = pos::U_BOARD_WIDTH * 2 - 4 - 4 - 1;
const I_DIAGONAL_SLICE_AMOUNT: isize = DIAGONAL_SLICE_AMOUNT as isize;

#[derive(Debug, Copy, Clone)]
pub struct Slice {
    pub length: u8,
    pub start_row: u8,
    pub start_col: u8,
    pub black_stones: u16,
    pub white_stones: u16,
    pub black_pattern_available: bool,
    pub white_pattern_available: bool,
}

impl Slice {

    pub const fn empty(length: u8, start_row: u8, start_col: u8) -> Self {
        Slice {
            length,
            start_row,
            start_col,
            black_stones: 0,
            white_stones: 0,
            black_pattern_available: false,
            white_pattern_available: false,
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
        };

        self.calculate_available_pattern_mut();
    }

    pub fn unset_mut(&mut self, color: Color, idx: u8) {
        let mask = !(0b1 << idx);
        match color {
            Color::Black => self.black_stones &= mask,
            Color::White => self.white_stones &= mask
        };

        self.calculate_available_pattern_mut();
    }

    pub fn stones<const C: Color>(&self) -> u16 {
        match C {
            Color::Black => self.black_stones,
            Color::White => self.white_stones
        }
    }

    pub fn stones_reversed<const C: Color>(&self) -> u16 {
        match C {
            Color::Black => self.white_stones,
            Color::White => self.black_stones
        }
    }

    pub fn pattern_available<const C: Color>(&self) -> bool {
        match C {
            Color::Black => self.black_pattern_available,
            Color::White => self.white_pattern_available
        }
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

    pub fn packed_slice<const C: Color>(&self) -> u64 {
        (C as u64) << 40 | (self.length as u64) << 32 | (self.white_stones as u64) << 16 | self.black_stones as u64
    }

    pub fn calculate_idx(&self, direction: Direction, pos: Pos) -> usize {
        match direction {
            Direction::Vertical => pos.row_usize(),
            Direction::Horizontal => pos.col_usize(),
            _ => pos.col_usize() - self.start_col as usize
        }
    }

    fn calculate_available_pattern_mut(&mut self) {
        // filter . . O . . . .
        // filter O X . . O X .
        // filter O . . . O . .
        // TODO: filter . X O O O X .
        self.black_pattern_available = self.black_stones.count_ones() > 1
            && self.black_stones & !(self.white_stones << 1) & !(self.white_stones >> 1) != 0
            && self.black_stones & ((self.black_stones << 2) | (self.black_stones << 1) | (self.black_stones >> 1) | (self.black_stones >> 2)) != 0;
        self.white_pattern_available = self.white_stones.count_ones() > 1
            && self.white_stones & !(self.black_stones << 1) & !(self.black_stones >> 1) != 0
            && self.white_stones & ((self.white_stones << 2) | (self.white_stones << 1) | (self.white_stones >> 1) | (self.white_stones >> 2)) != 0;
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
                Slice::empty(pos::BOARD_WIDTH, idx as u8, 0)
            ),
            vertical_slices: std::array::from_fn(|idx|
                Slice::empty(pos::BOARD_WIDTH, 0, idx as u8)
            ),
            ascending_slices: std::array::from_fn(|idx| {
                let seq_num = idx as isize + 5 - pos::I_BOARD_WIDTH;
                Slice::empty(
                    (seq_num.abs() - pos::I_BOARD_WIDTH).unsigned_abs() as u8,
                    max(0, seq_num.neg()) as u8,
                    max(0, seq_num) as u8
                )
            }),
            descending_slices: std::array::from_fn(|idx| {
                let seq_num = idx as isize + 5 - pos::I_BOARD_WIDTH;
                Slice::empty(
                    (seq_num.abs() - pos::I_BOARD_WIDTH).unsigned_abs() as u8,
                    pos::BOARD_WIDTH - 1 - max(0, seq_num.neg()) as u8,
                    max(0, seq_num) as u8
                )
            })
        }
    }

}

impl Slices {

    pub fn set_mut(&mut self, color: Color, pos: Pos) {
        self.horizontal_slices[pos.row_usize()].set_mut(color, pos.col());
        self.vertical_slices[pos.col_usize()].set_mut(color, pos.row());

        if let Some(ascending_slice) = Self::ascending_slice_idx(pos).map(|idx|
            &mut self.ascending_slices[idx]
        ) {
            ascending_slice.set_mut(color, pos.col() - ascending_slice.start_col)
        }

        if let Some(descending_slice) = Self::descending_slice_idx(pos).map(|idx|
            &mut self.descending_slices[idx]
        ) {
            descending_slice.set_mut(color, pos.col() - descending_slice.start_col)
        }
    }

    pub fn access_slice(&self, direction: Direction, pos: Pos) -> &Slice {
        match direction {
            Direction::Horizontal => &self.horizontal_slices[pos.row_usize()],
            Direction::Vertical => &self.vertical_slices[pos.col_usize()],
            Direction::Ascending => &self.ascending_slices[Self::calculate_ascending_slice_idx(pos) as usize],
            Direction::Descending => &self.descending_slices[Self::calculate_descending_slice_idx(pos) as usize],
        }
    }
    
    fn calculate_ascending_slice_idx(pos: Pos) -> isize {
        // y = x + b, b = y - x (reversed row sequence)
        I_DIAGONAL_SLICE_AMOUNT - (pos.row() as isize - pos.col() as isize + pos::I_BOARD_WIDTH - 4)
    }

    pub fn ascending_slice_idx(pos: Pos) -> Option<usize> {
        let idx = Self::calculate_ascending_slice_idx(pos);
        (0 .. I_DIAGONAL_SLICE_AMOUNT).contains(&idx)
            .then_some(idx as usize)
    }
    
    fn calculate_descending_slice_idx(pos: Pos) -> isize {
        // y = -x + 15 + b, b = y + x - 15
        pos.row() as isize + pos.col() as isize - 4
    }

    pub fn descending_slice_idx(pos: Pos) -> Option<usize> {
        let idx = Self::calculate_descending_slice_idx(pos);
        (0 .. I_DIAGONAL_SLICE_AMOUNT).contains(&idx)
            .then_some(idx as usize)
    }
}
