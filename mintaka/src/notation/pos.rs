use crate::notation::rule;

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub struct Pos(u8);

pub fn cartesian_to_index(row: u8, col: u8) -> u8 {
    row * rule::BOARD_WIDTH + col
}

pub fn cartesian_to_index_usize(row: usize, col: usize) -> usize {
    row * rule::U_BOARD_WIDTH + col
}

impl Pos {

    pub fn from_index(index: u8) -> Self {
        Pos(index)
    }

    pub fn from_index_usize(index: usize) -> Self {
        Pos(index as u8)
    }

    pub fn from_cartesian(row: u8, col: u8) -> Self {
        Pos(cartesian_to_index(row, col))
    }

    pub fn from_cartesian_usize(row: usize, col: usize) -> Self {
        Pos(cartesian_to_index_usize(row, col) as u8)
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
        self.0 / rule::BOARD_WIDTH
    }

    pub fn row_usize(&self) -> usize {
        self.row() as usize
    }

    pub fn col(&self) -> u8 {
        self.0 % rule::BOARD_WIDTH
    }

    pub fn col_usize(&self) -> usize {
        self.col() as usize
    }

    fn mask_col(&self) -> u8 {
        0b1 >> self.col()
    }

    fn reverse_mask_col(&self) -> u8 {
        !self.mask_col()
    }

}
