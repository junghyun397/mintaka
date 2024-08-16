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

const DIAGONAL_SLICE_AMOUNT: usize = rule::U_BOARD_WIDTH * 2 - 4 - 4 - 1;
const SLICE_AMOUNT: usize = rule::U_BOARD_WIDTH * 2 + DIAGONAL_SLICE_AMOUNT * 2;

const I_DIAGONAL_SLICE_AMOUNT: isize = DIAGONAL_SLICE_AMOUNT as isize;

#[derive(Debug, Copy, Clone)]
pub struct Slice {
    pub length: u8,
    pub start_pos: Pos,
    pub black_stones: u16,
    pub white_stones: u16,
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

impl Slices {

    pub fn empty() -> Self {
        Slices {
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

    pub fn access_slice(&self, pos: Pos, direction: Direction) -> Option<&Slice> {
        match direction {
            Direction::Horizontal => Some(&self.horizontal_slices[pos.row() as usize]),
            Direction::Vertical => Some(&self.vertical_slices[pos.col() as usize]),
            Direction::Ascending => { // y = x + b, b = y - x (reversed row sequence)
                let idx = I_DIAGONAL_SLICE_AMOUNT - (pos.row() as isize - pos.col() as isize + rule::I_BOARD_WIDTH - 4);
                match idx {
                    0 .. I_DIAGONAL_SLICE_AMOUNT => Some(&self.ascending_slices[idx as usize]),
                    _ => None
                }
            },
            Direction::Descending => { // y = -x + 15 + b, b = y + x - 15
                let idx = pos.row() as isize + pos.col() as isize - 4;
                match idx {
                    0 .. I_DIAGONAL_SLICE_AMOUNT => Some(&self.descending_slices[idx as usize]),
                    _ => None
                }
            }
        }
    }

    pub fn build_slice_set(&self, pos: Pos) -> SliceSet {
        SliceSet {
            horizontal_slice: self.access_slice(pos, Direction::Horizontal).unwrap().clone(),
            vertical_slice: self.access_slice(pos, Direction::Vertical).unwrap().clone(),
            ascending_slice: self.access_slice(pos, Direction::Ascending).map(|x| x.clone()),
            descending_slice: self.access_slice(pos, Direction::Descending).map(|x| x.clone()),
        }
    }

}
