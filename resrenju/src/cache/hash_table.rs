use crate::notation::rule;

pub const EMPTY_HASH: u64 = 0;

pub const TABLE_SIZE: usize = rule::BOARD_SIZE as usize * 2;
pub const TABLE: [u64; TABLE_SIZE] = [0; TABLE_SIZE]; // TODO
