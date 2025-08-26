use rusty_renju::notation::value::Score;

pub const ASPIRATION_INITIAL_DELTA: Score = 16;
pub const MAX_PLY: usize = 128;

pub type Depth = i32;

pub trait Depths {
    const HARD_LIMIT: Depth = MAX_PLY as Depth - 10;
}

impl Depths for Depth {}
