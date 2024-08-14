use std::cmp::max;
use std::ops::Neg;
use crate::notation::color::Color;
use crate::notation::pos::Pos;
use crate::notation::rule;

#[derive(Debug)]
pub enum Direction {
    Horizontal = 0,
    Vertical = 1,
    Ascending = 2,
    Descending = 3
}

const DIAGONAL_SLICE_AMOUNT: usize = rule::BOARD_WIDTH as usize * 2 - 4 - 4 - 1;
const SLICE_AMOUNT: usize = rule::BOARD_WIDTH as usize * 2 + DIAGONAL_SLICE_AMOUNT * 2;

#[derive(Debug)]
pub struct Slice {
    pub direction: Direction,
    pub length: u8,
    pub start_pos: Pos,
    pub black_stones: u16,
    pub white_stones: u16,
}

impl Slice {

    pub fn empty(direction: Direction, length: u8, start_pos: Pos) -> Self {
        Slice {
            direction,
            length,
            start_pos,
            black_stones: 0,
            white_stones: 0
        }
    }

    pub fn set(&self, idx: u8) -> Self {
        todo!()
    }

    pub fn unset(&self, idx: u8) -> Self {
        todo!()
    }

    pub fn set_mut(&mut self, idx: u8) {
        todo!()
    }

    pub fn unset_mut(&mut self, idx: u8) {
        todo!()
    }

    pub fn black_stone_at(&self, idx: u8) -> bool {
        self.black_stones & 0b1000_0000_0000_0000 >> idx == 0xb1
    }

    pub fn white_stone_at(&self, idx: u8) -> bool {
        self.white_stones & 0b1000_0000_0000_0000 >> idx == 0xb1
    }

    pub fn stone_at(&self, color: Color, idx: u8) -> bool {
        match color {
            Color::Black => self.black_stone_at(idx),
            Color::White => self.white_stone_at(idx)
        }
    }

}

pub struct Slices {
    pub horizontal_slices: [Slice; rule::BOARD_WIDTH as usize],
    pub vertical_slices: [Slice; rule::BOARD_WIDTH as usize],
    pub ascending_slices: [Slice; DIAGONAL_SLICE_AMOUNT],
    pub descending_slices: [Slice; DIAGONAL_SLICE_AMOUNT]
}

pub struct SliceSet {
    pub horizontal_slice: Slice,
    pub vertical_slice: Slice,
    pub ascending_slice: Slice,
    pub descending_slice: Slice
}

impl Slices {

    pub fn empty() -> Self {
        Slices {
            horizontal_slices: std::array::from_fn(|idx|
                Slice::empty(Direction::Horizontal, rule::BOARD_WIDTH, Pos::from_cartesian(idx as u8, 0))
            ),
            vertical_slices: std::array::from_fn(|idx|
                Slice::empty(Direction::Horizontal, rule::BOARD_WIDTH, Pos::from_cartesian(0, idx as u8))
            ),
            ascending_slices: std::array::from_fn(|idx| {
                let seq_num = idx as isize + 5 - rule::BOARD_WIDTH as isize;
                Slice::empty(
                    Direction::Ascending,
                    (seq_num.abs() - rule::BOARD_WIDTH as isize).abs() as u8,
                    Pos::from_cartesian(max(0, seq_num.neg()) as u8, max(0, seq_num) as u8)
                )
            }),
            descending_slices: std::array::from_fn(|idx| {
                let seq_num = idx as isize + 5 - rule::BOARD_WIDTH as isize;
                Slice::empty(
                    Direction::Descending,
                    (seq_num.abs() - rule::BOARD_WIDTH as isize).abs() as u8,
                    Pos::from_cartesian(
                        rule::BOARD_WIDTH-1 - max(0, seq_num.neg()) as u8,
                        max(0, seq_num) as u8
                    )
                )
            })
        }
    }

    pub fn access_slice(&self, pos: Pos, direction: Direction) -> &Slice {
        todo!()
    }

    pub fn build_slice_set(&self, pos: Pos) -> SliceSet {
        todo!()
    }

}
