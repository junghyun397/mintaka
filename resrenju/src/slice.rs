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
        let mask = 0b1000_0000_0000_0000 >> idx;

        let (black_stones, white_stones) = if color == Color::Black {
            (self.black_stones | mask, self.white_stones)
        } else {
            (self.black_stones, self.white_stones | mask)
        };

        Slice {
            length: self.length,
            start_pos: self.start_pos,
            black_stones,
            white_stones
        }
    }

    pub fn unset(&self, color: Color, idx: u8) -> Self {
        let mask = !(0b1000_0000_0000_0000 >> idx);

        let (black_stones, white_stones) = if color == Color::Black {
            (self.black_stones & mask, self.white_stones)
        } else {
            (self.black_stones, self.white_stones & mask)
        };

        Slice {
            length: self.length,
            start_pos: self.start_pos,
            black_stones,
            white_stones
        }
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

    pub fn set(&self, color: Color, pos: Pos) -> Self {
        self.slice_op(
            pos,
            |vertical_slice| {
               vertical_slice.set(color, pos.row())
            },
            |horizontal_slice| {
                horizontal_slice.set(color, pos.col())
            },
            |ascending_slice| {
                ascending_slice.set(color, pos.col() - ascending_slice.start_pos.col())
            },
            |descending_slice| {
                descending_slice.set(color, pos.col() - descending_slice.start_pos.col())
            }
        )
    }

    pub fn unset(&self, color: Color, pos: Pos) -> Self {
        self.slice_op(
            pos,
            |vertical_slice| {
                vertical_slice.unset(color, pos.row())
            },
            |horizontal_slice| {
                horizontal_slice.unset(color, pos.col())
            },
            |ascending_slice| {
                ascending_slice.unset(color, pos.col() - ascending_slice.start_pos.col())
            },
            |descending_slice| {
                descending_slice.unset(color, pos.col() - descending_slice.start_pos.col())
            }
        )
    }

    #[inline(always)]
    fn slice_op<F1, F2, F3, F4>(
        &self, pos: Pos,
        vertical_op: F1,
        horizontal_op: F2,
        ascending_op: F3,
        descending_op: F4
    ) -> Self where
        F1: FnOnce(&Slice) -> Slice,
        F2: FnOnce(&Slice) -> Slice,
        F3: FnOnce(&Slice) -> Slice,
        F4: FnOnce(&Slice) -> Slice,
    {
        let mut vertical_slices = self.vertical_slices.clone();
        vertical_slices[pos.row() as usize] = vertical_op(&self.vertical_slices[pos.row() as usize]);

        let mut horizontal_slices = self.horizontal_slices.clone();
        horizontal_slices[pos.col() as usize] = horizontal_op(&self.horizontal_slices[pos.col() as usize]);

        let mut ascending_slices = self.ascending_slices.clone();
        let idx = I_DIAGONAL_SLICE_AMOUNT - (pos.row() as isize - pos.col() as isize + rule::I_BOARD_WIDTH - 4);
        if 0 <= idx && idx < I_DIAGONAL_SLICE_AMOUNT {
            ascending_slices[idx as usize] = ascending_op(&self.ascending_slices[idx as usize]);
        }

        let mut descending_slices = self.descending_slices.clone();
        let idx = pos.row() as isize + pos.col() as isize - 4;
        if 0 <= idx && idx < I_DIAGONAL_SLICE_AMOUNT {
            descending_slices[idx as usize] = descending_op(&self.descending_slices[idx as usize]);
        }

        Slices {
            vertical_slices,
            horizontal_slices,
            ascending_slices,
            descending_slices
        }
    }

    pub fn set_mut<T>(&mut self, color: Color, pos: Pos) {
        self.slice_op_mut(
            pos,
            |vertical_slice| {
               vertical_slice.set_mut(color, pos.row());
            },
            |horizontal_slice| {
                horizontal_slice.set_mut(color, pos.col());
            },
            |ascending_slice| {
                ascending_slice.set_mut(color, pos.col() - ascending_slice.start_pos.col());
            },
            |descending_slice| {
                descending_slice.set_mut(color, pos.col() - descending_slice.start_pos.col());
            }
        )
    }

    fn unset_mut(&mut self, color: Color, pos: Pos) {
        self.slice_op_mut(
            pos,
            |vertical_slice| {
               vertical_slice.unset_mut(color, pos.row());
            },
            |horizontal_slice| {
                horizontal_slice.unset_mut(color, pos.col());
            },
            |ascending_slice| {
                ascending_slice.unset_mut(color, pos.col() - ascending_slice.start_pos.col());
            },
            |descending_slice| {
                descending_slice.unset_mut(color, pos.col() - descending_slice.start_pos.col());
            }
        )
    }

    #[inline(always)]
    fn slice_op_mut<F1, F2, F3, F4>(
        &mut self, pos: Pos,
        vertical_op: F1,
        horizontal_op: F2,
        ascending_op: F3,
        descending_op: F4
    ) where
        F1: FnOnce(&mut Slice) -> (),
        F2: FnOnce(&mut Slice) -> (),
        F3: FnOnce(&mut Slice) -> (),
        F4: FnOnce(&mut Slice) -> (),
    {
        vertical_op(&mut self.vertical_slices[pos.row() as usize]);
        horizontal_op(&mut self.horizontal_slices[pos.col() as usize]);

        let idx = I_DIAGONAL_SLICE_AMOUNT - (pos.row() as isize - pos.col() as isize + rule::I_BOARD_WIDTH - 4);
        if 0 <= idx && idx < I_DIAGONAL_SLICE_AMOUNT {
            ascending_op(&mut self.ascending_slices[idx as usize])
        }

        let idx = pos.row() as isize + pos.col() as isize - 4;
        if 0 <= idx && idx < I_DIAGONAL_SLICE_AMOUNT {
            descending_op(&mut self.descending_slices[idx as usize]);
        }
    }

    pub fn access_slice(&self, d: Direction, pos: Pos) -> Option<&Slice> {
        match d {
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

}
