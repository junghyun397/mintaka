use crate::memo::transposition_table::TranspositionTable;
use mintaka::board::Board;
use mintaka::memo::slice_pattern_memo::SlicePatternMemo;
use mintaka::notation::pos::Pos;

pub trait NodeType {
    const IS_PV: bool;
    const IS_ROOT: bool;
}

pub fn search(
    board: &mut Board, tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo
) -> (i16, Option<Pos>) {
    (0, None)
}
