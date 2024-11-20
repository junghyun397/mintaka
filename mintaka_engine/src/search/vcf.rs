use crate::memo::transposition_table::TranspositionTable;
use mintaka::board::Board;
use mintaka::memo::slice_pattern_memo::SlicePatternMemo;
use mintaka::notation::color::Color;
use mintaka::notation::pos;
use mintaka::notation::pos::{Pos, INVALID_POS};

pub fn vcf(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board, max_depth: u8
) -> Option<Vec<Pos>> {
    match board.player_color {
        Color::Black => vcf_by_color::<{ Color::Black }>(tt, memo, board, max_depth, 0),
        Color::White => vcf_by_color::<{ Color::White }>(tt, memo, board, max_depth, 0)
    }
}

// Depth-First Search(DFS)
fn vcf_by_color<const C: Color>(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board, max_depth: u8, depth: u8
) -> Option<Vec<Pos>> {
    if depth > max_depth {
        return None;
    }

    for idx in 0 .. pos::BOARD_SIZE {
        let pattern = &board.patterns.field[idx];
        let unit = match C {
            Color::Black => pattern.black_unit,
            Color::White => pattern.white_unit
        };

        if !unit.has_four()
            || (C == Color::Black && pattern.is_forbidden())
        {
            continue;
        }

        let four_pos = Pos::from_index(idx as u8);

        if unit.has_three() // three-four fork
            || unit.has_open_four()
            || (C == Color::White && unit.has_fours())
        {
            return Some(vec![four_pos])
        }

        board.set_mut(memo, four_pos);

        let maybe_vcf = vcf_defend::<C>(tt, memo, board, max_depth, depth + 1);

        board.unset_mut(memo, four_pos);

        if let Some(vcf) = maybe_vcf {
            let mut new_vcf = vcf;
            new_vcf.push(four_pos);
            return Some(new_vcf);
        }
    }

    None
}

fn vcf_defend<const C: Color>(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board, max_depth: u8, depth: u8
) -> Option<Vec<Pos>> {
    let defend_pos = {
        let mut defend_pos = INVALID_POS;

        for idx in 0 .. pos::BOARD_SIZE {
            let pattern = &board.patterns.field[idx];
            let unit = match C {
                Color::Black => pattern.black_unit,
                Color::White => pattern.white_unit
            };

            if !unit.has_five() {
                continue;
            }

            if C == Color::White && pattern.is_forbidden() { // trap vcf
                return Some(vec![]);
            }

            defend_pos = Pos::from_index(idx as u8);
        }

        assert_ne!(defend_pos, INVALID_POS);
        defend_pos
    };

    board.set_mut(memo, defend_pos);

    let maybe_vcf = vcf_by_color::<C>(tt, memo, board, max_depth, depth + 1);

    board.unset_mut(memo, defend_pos);

    maybe_vcf.map(|vcf| {
        let mut new_vcf = vcf;
        new_vcf.push(defend_pos);
        new_vcf
    })
}
