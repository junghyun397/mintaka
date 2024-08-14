pub const BOARD_WIDTH: u8 = 15;
pub const BOARD_SIZE: u8 = BOARD_WIDTH * BOARD_WIDTH;

pub enum RuleKind {
    FiveInARow = 0,
    SimplifiedRenju = 1,
    Renju = 2
}
