use rusty_renju::notation::score::Score;

pub const MAX_PLY: usize = 128;
pub const MAX_PLY_SLOTS: usize = MAX_PLY + 1;

pub type Depth = i32;

pub trait Depths {
    const PLY_LIMIT: Depth = MAX_PLY as Depth;
}

impl Depths for Depth {}

pub const ASPIRATION_INITIAL_DELTA: Score = 16;

pub const LMR_BASE: f64 = 8.0;
pub const LMR_DIV: f64 = 64.0;
