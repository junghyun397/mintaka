use rusty_renju::notation::score::Score;

pub const ASPIRATION_INITIAL_DELTA: Score = 16;
pub const MAX_PLY: usize = 128;

pub type Depth = i32;

pub trait Depths {
    const PLY_LIMIT: Depth = MAX_PLY as Depth;
}

impl Depths for Depth {}
