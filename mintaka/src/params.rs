use crate::value::Depth;
use rusty_renju::notation::score::Score;

pub const ASPIRATION_DELTA_BASE: Score = 8;
pub const ASPIRATION_DELTA_DIV: Score = 8192;

pub const LMR_BASE: f64 = 0.8;
pub const LMR_DIV: f64 = 2.4;

pub const LMP_BASE: usize = 2;
pub const LMP_DIV_IMPROVING: f64 = 1.0;
pub const LMP_DIV_NON_IMPROVING: f64 = 2.0;

pub const FP_BASE: Depth = 100;
pub const FP_MUL: Depth = 42;

pub const HISTORY_TABLE_AGEING_MUL: f64 = 0.75;
