use crate::memo::transposition_table::TranspositionTable;
use mintaka::board::Board;
use mintaka::memo::slice_pattern_memo::SlicePatternMemo;
use mintaka::notation::color::Color;
use mintaka::notation::pos::Pos;

pub fn vct_sequence(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board, max_depth: usize
) -> Option<Vec<Pos>> {
    match board.player_color {
        Color::Black => try_vct::<{ Color::Black }>(tt, memo, board, max_depth, false),
        Color::White => try_vct::<{ Color::White }>(tt, memo, board, max_depth, false)
    }.map(|mut result| {
        result.reverse();
        result
    })
}

// Proof-Number Search(PNS), https://www.chessprogramming.org/Proof-Number_Search
fn try_vct<const C: Color>(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board, max_depth: usize, opponent_has_open_four: bool,
) -> Option<Vec<Pos>> {
    None
}

fn defend_vct<const C: Color>(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board, max_depth: u8, depth: u8, opponent_has_open_four: bool,
) -> Option<Vec<Pos>> {
    None
}
