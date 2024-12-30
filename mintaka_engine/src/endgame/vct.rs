use crate::memo::transposition_table::TranspositionTable;
use rusty_renju::board::Board;
use rusty_renju::memo::slice_pattern_memo::SlicePatternMemo;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::Pos;

pub fn vct_sequence(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board, max_depth: u8
) -> Option<Vec<Pos>> {
    match board.player_color {
        Color::Black => try_vct::<{ Color::Black }>(tt, memo, board, max_depth, 0),
        Color::White => try_vct::<{ Color::White }>(tt, memo, board, max_depth, 0)
    }.map(|mut result| {
        result.reverse();
        result
    })
}

// Proof-Number Search(PNS), https://www.chessprogramming.org/Proof-Number_Search
pub fn try_vct<const C: Color>(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo, board: &mut Board,
    max_depth: u8, depth: u8
) -> Option<Vec<Pos>> {
    None
}

pub fn defend_by_vcf<const C: Color>(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo, board: &mut Board,
    max_depth: u8, depth: u8
) -> bool {
    todo!()
}
