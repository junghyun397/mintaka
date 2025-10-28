use rusty_renju::notation::score::Score;

pub const MAX_PLY: usize = 128;
pub const MAX_PLY_SLOTS: usize = MAX_PLY + 1;

pub type Depth = i32;

pub trait Depths {
    const PLY_LIMIT: Depth = MAX_PLY as Depth;
}

impl Depths for Depth {}

pub const ASPIRATION_INITIAL_DELTA: Score = 16;

pub const LMR_BASE: f64 = 0.8;
pub const LMR_DIV: f64 = 2.4;

pub const LMP_BASE: usize = 2;
pub const LMP_DIV_IMPROVING: f64 = 1.0;
pub const LMP_DIV_NON_IMPROVING: f64 = 2.0;

pub const FP_BASE: Depth = 100;
pub const FP_MUL: Depth = 42;

pub const HISTORY_TABLE_AGEING_MUL: f64 = 0.75;
