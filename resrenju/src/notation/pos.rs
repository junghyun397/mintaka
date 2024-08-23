use crate::notation::rule;

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub struct Pos(u8);

impl Pos {

    pub fn from_cartesian(row: u8, col: u8) -> Self {
        Pos(row * rule::BOARD_WIDTH + col)
    }

    pub fn to_cartesian(&self) -> (u8, u8) {
        (self.row(), self.col())
    }

    pub fn from_index(index: u8) -> Self {
        Pos(index)
    }

    pub fn idx(&self) -> u8 {
        self.0
    }

    pub fn row(&self) -> u8 {
        self.0 / rule::BOARD_WIDTH
    }

    pub fn col(&self) -> u8 {
        self.0 % rule::BOARD_WIDTH
    }

    pub fn mask_col(&self) -> u8 {
        0b1 >> self.col()
    }

    pub fn reverse_mask_col(&self) -> u8 {
        !self.mask_col()
    }

}
