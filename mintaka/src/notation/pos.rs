use crate::notation::direction::Direction;
use crate::utils::str_utils::u8_from_str;

pub const BOARD_WIDTH: u8 = 15;
pub const BOARD_SIZE: usize = U_BOARD_WIDTH * U_BOARD_WIDTH;

pub const U_BOARD_WIDTH: usize = BOARD_WIDTH as usize;
pub const I_BOARD_WIDTH: isize = BOARD_WIDTH as isize;

pub const U8_BOARD_SIZE: u8 = BOARD_SIZE as u8;

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

#[macro_export] macro_rules! check_cartesian_bound {
    ($row:expr,$col:expr) => ($row < 15 && $col < 15);
}

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub struct Pos(u8);

impl Pos {

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

    pub const fn offset(&self, row: u8, col: u8) -> Pos {
        Self::from_cartesian(self.row() + row, self.col() + col)
    }
    
    pub const fn directional_offset(&self, direction: Direction, offset: i16) -> Pos {
        match direction {
            Direction::Vertical => Self::from_cartesian((self.row() as i16 + offset) as u8, self.col()),
            Direction::Horizontal => Self::from_cartesian(self.row(), (self.col() as i16 + offset) as u8),
            Direction::Ascending =>
                Self::from_cartesian((self.row() as i16 + offset) as u8, (self.col() as i16 + offset) as u8),
            Direction::Descending =>
                Self::from_cartesian((self.row() as i16 - offset) as u8, (self.col() as i16 + offset) as u8)
        }
    }

}

pub const INVALID_POS: Pos = Pos::from_index(u8::MAX);
