use crate::notation::direction::Direction;
use crate::utils::str_utils::u8_from_str;

pub const BOARD_WIDTH: u8 = 15;
pub const BOARD_SIZE: usize = U_BOARD_WIDTH * U_BOARD_WIDTH;
pub const BOARD_BOUND: usize = BOARD_SIZE - 1;

pub const U_BOARD_WIDTH: usize = BOARD_WIDTH as usize;
pub const I_BOARD_WIDTH: isize = BOARD_WIDTH as isize;

pub const U8_BOARD_SIZE: u8 = BOARD_SIZE as u8;
pub const U8_BOARD_BOUND: u8 = BOARD_BOUND as u8;

pub const CENTER: Pos = Pos::from_index(U8_BOARD_SIZE / 2);
pub const CENTER_ROW_COL: u8 = CENTER.col();

#[macro_export] macro_rules! cartesian_to_index {
    ($row:expr,$col:expr) => ($row * 15 + $col);
}

#[macro_export] macro_rules! index_to_cartesian {
    ($idx:expr) => (($idx / 15, $idx % 15));
}

#[macro_export] macro_rules! index_to_row {
    ($idx:expr) => ($idx / 15)
}

#[macro_export] macro_rules! index_to_col {
    ($idx:expr) => ($idx % 15);
}

const STEP_TABLE: [isize; 4] = [1, BOARD_WIDTH as isize, BOARD_WIDTH as isize + 1, -(BOARD_WIDTH as isize - 1)];

#[inline(always)]
pub const fn step_idx_usize<const D: Direction>(idx: usize) -> usize {
    match D {
        Direction::Horizontal => idx + 1,
        Direction::Vertical => idx + U_BOARD_WIDTH,
        Direction::Ascending => idx + U_BOARD_WIDTH + 1,
        Direction::Descending => idx - U_BOARD_WIDTH + 1
    }
}

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub struct Pos(u8);

impl Pos {

    pub const INVALID: Self = Self(u8::MAX);

    pub const fn from_index(index: u8) -> Self {
        Self(index)
    }

    pub const fn from_cartesian(row: u8, col: u8) -> Self {
        Self(cartesian_to_index!(row, col))
    }

    pub const fn from_str_unchecked(source: &str) -> Pos {
        let row = u8_from_str(source, 1) - 1;
        let col = source.as_bytes()[0] - b'a';

        Self::from_cartesian(row, col)
    }

    pub const fn to_cartesian(&self) -> (u8, u8) {
        index_to_cartesian!(self.0)
    }

    pub const fn idx(&self) -> u8 {
        self.0
    }

    pub const fn idx_usize(&self) -> usize {
        self.0 as usize
    }

    pub const fn row(&self) -> u8 {
        index_to_row!(self.0)
    }

    pub const fn row_usize(&self) -> usize {
        self.row() as usize
    }

    pub const fn col(&self) -> u8 {
        index_to_col!(self.0)
    }

    pub const fn col_usize(&self) -> usize {
        self.col() as usize
    }

    pub const fn offset(&self, offset_row: isize, offset_col: isize) -> Option<Pos> {
        let row = self.row() as isize + offset_row;
        let col = self.col() as isize + offset_col;

        if row >= 0 && row < I_BOARD_WIDTH && col >= 0 && col < I_BOARD_WIDTH {
            Some(Pos::from_cartesian(row as u8, col as u8))
        } else {
            None
        }
    }

    pub const fn offset_unchecked(&self, offset_row: isize, offset_col: isize) -> Pos {
        Pos::from_cartesian(
            self.row_usize().saturating_add_signed(offset_row) as u8,
            self.col_usize().saturating_add_signed(offset_col) as u8
        )
    }

    pub const fn directional_offset_unchecked(&self, direction: Direction, offset: isize) -> Pos {
        Self::from_index((self.0 as isize + (STEP_TABLE[direction as usize] * offset)) as u8)
    }

}
