use crate::memo::transposition_table::TranspositionTable;
use mintaka::board::Board;
use mintaka::memo::slice_pattern_memo::SlicePatternMemo;
use mintaka::notation::color::Color;
use mintaka::notation::pos;
use mintaka::notation::pos::{Pos, INVALID_POS};

pub fn vcf(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board, max_depth: usize
) -> Option<Vec<Pos>> {
    match board.player_color {
        Color::Black => find_vcf_solution::<{ Color::Black }>(tt, memo, board, max_depth),
        Color::White => find_vcf_solution::<{ Color::White }>(tt, memo, board, max_depth)
    }
}

// Iterative Deepening Search(IDS), https://www.chessprogramming.org/Iterative_Deepening
pub fn find_vcf_solution<const C: Color>(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board, max_depth: usize
) -> Option<Vec<Pos>> {
    let mut four_queue: [Pos; 16] = [INVALID_POS; 16];
    let mut four_top: usize = 0;

    let mut critical_point = false;

    for idx in 0 .. pos::BOARD_SIZE {
        let pattern = board.patterns.field[idx].clone();
        let unit = match C {
            Color::Black => pattern.black_unit,
            Color::White => pattern.white_unit
        };

        if C == Color::Black && pattern.is_forbidden() {
            continue;
        }

        if unit.has_four() {
            four_queue[four_top] = Pos::from_index(idx as u8);
            four_top += 1;
        }
    }

    for queue_idx in 0 .. four_top {
        let next_pos = four_queue[queue_idx];

        board.set_mut(memo, next_pos);
        let vcf_result = find_vcf_solution::<C>(tt, memo, board, max_depth);
        board.unset_mut(memo, next_pos);

        if vcf_result.is_some() {
            vcf_result?.push(next_pos);
            return vcf_result;
        }
    }

    None
}


