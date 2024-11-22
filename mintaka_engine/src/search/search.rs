use crate::memo::transposition_table::TranspositionTable;
use crate::memo::tt_entry::Score;
use mintaka::board::Board;
use mintaka::memo::slice_pattern_memo::SlicePatternMemo;
use mintaka::notation::pos::Pos;

pub trait NodeType {
    const IS_PV: bool;
    const IS_ROOT: bool;
}

// Principal Variation Search(PVS), https://www.chessprogramming.org/Principal_Variation_Search
pub fn search(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board
) -> (Score, Option<Pos>) {
    (0, None)
}
