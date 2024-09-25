use crate::utils::str_utils::u8_from_str;

pub const BOARD_WIDTH: u8 = 15;
pub const BOARD_SIZE: usize = U_BOARD_WIDTH * U_BOARD_WIDTH;

pub const U_BOARD_WIDTH: usize = BOARD_WIDTH as usize;
pub const I_BOARD_WIDTH: isize = BOARD_WIDTH as isize;

pub const U8_BOARD_SIZE: u8 = BOARD_SIZE as u8;

pub const CENTER: Pos = Pos::from_index(U8_BOARD_SIZE / 2);

#[macro_export] macro_rules! cartesian_to_index {
    ($row:expr,$col:expr) => ($row * 15 + $col);
}

#[macro_export] macro_rules! index_to_cartesian {
    ($idx:expr) => (($idx / 15, $idx % 15));
}

#[macro_export] macro_rules! check_cartesian_bound {
    ($row:expr,$col:expr) => ($row < 15 && $col < 15);
}

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub struct Pos(u8);

impl Pos {

    pub const fn from_index(index: u8) -> Self {
        Pos(index)
    }

    pub const fn from_cartesian(row: u8, col: u8) -> Self {
        Pos(cartesian_to_index!(row, col))
    }

    pub fn to_cartesian(&self) -> (u8, u8) {
        (self.row(), self.col())
    }

    pub fn idx(&self) -> u8 {
        self.0
    }

    pub fn idx_usize(&self) -> usize {
        self.0 as usize
    }

    pub fn row(&self) -> u8 {
        self.0 / BOARD_WIDTH
    }

    pub fn row_usize(&self) -> usize {
        self.row() as usize
    }

    pub fn col(&self) -> u8 {
        self.0 % BOARD_WIDTH
    }

    pub fn col_usize(&self) -> usize {
        self.col() as usize
    }

    pub fn mask_col(&self) -> u8 {
        0b1 >> self.col()
    }

    pub fn reverse_mask_col(&self) -> u8 {
        !self.mask_col()
    }

}

pub const fn pos_unchecked(source: &str) -> Pos {
    let row = u8_from_str(source, 1);
    let col = source.as_bytes()[0] - b'a' as u8;

    Pos::from_cartesian(row, col)
}
