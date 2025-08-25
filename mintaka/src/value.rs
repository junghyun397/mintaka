use rusty_renju::notation::value::{Depth, Score};

pub const ASPIRATION_INITIAL_DELTA: Score = 16;
pub const MAX_PLY: usize = 128;
pub const MAX_DEPTH: Depth = MAX_PLY as Depth;
