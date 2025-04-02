use rusty_renju::notation::value::Score;

pub const TT_MOVE: Score = Score::MAX - 500;
pub const KILLER_MOVE: Score = Score::MAX - 500;
pub const COUNTER_MOVE: Score = Score::MAX - 500;
