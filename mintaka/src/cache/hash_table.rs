use crate::notation::pos;

pub const EMPTY_HASH: u64 = 0;

pub const TABLE_SIZE: usize = pos::BOARD_SIZE * 2;
pub const TABLE: [u64; TABLE_SIZE] = [0; TABLE_SIZE]; // TODO
