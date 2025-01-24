use crate::value::Score;
use rusty_renju::notation::pos::Pos;

#[derive(Copy, Clone)]
pub struct HistoryTableEntry {
    score: Score,
    best_move: Pos,
    counter_move: Pos
}

#[derive(Clone)]
pub struct HistoryTable {
}
