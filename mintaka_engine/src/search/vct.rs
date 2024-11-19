use crate::memo::transposition_table::TranspositionTable;
use mintaka::board::Board;
use mintaka::memo::slice_pattern_memo::SlicePatternMemo;
use mintaka::notation::color::Color;
use mintaka::notation::pos::Pos;

pub fn vct(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board, max_depth: usize
) -> Option<Vec<Pos>> {
    match board.player_color {
        Color::Black => vct_by_color::<{ Color::Black }>(tt, memo, board, max_depth),
        Color::White => vct_by_color::<{ Color::White }>(tt, memo, board, max_depth)
    }
}

// Proof-Number Search(PNS), https://www.chessprogramming.org/Proof-Number_Search
fn vct_by_color<const C: Color>(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board, max_depth: usize
) -> Option<Vec<Pos>> {
    None
}
