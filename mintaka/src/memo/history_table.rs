use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::value::Score;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct HistoryTableEntry {
    score: Score,
    best_move: Pos,
    counter_move: Pos
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HistoryTable {
}
