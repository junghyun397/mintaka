use crate::memo::transposition_table::TranspositionTable;
use mintaka::board::Board;
use mintaka::memo::slice_pattern_memo::SlicePatternMemo;
use mintaka::notation::color::Color;
use mintaka::notation::pos;
use mintaka::notation::pos::{Pos, INVALID_POS};

pub fn vcf(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board, max_depth: u8
) -> i16 {
    todo!()
}

pub fn vcf_sequence(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board, max_depth: u8
) -> Option<Vec<Pos>> {
    match board.player_color {
        Color::Black => try_vcf_v2::<{ Color::Black }>(tt, memo, board, max_depth, 0, false),
        Color::White => try_vcf_v2::<{ Color::White }>(tt, memo, board, max_depth, 0, false)
    }.map(|mut result| {
        result.reverse();
        result
    })
}

fn try_vcf_v2<const C: Color>(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board, max_depth: u8, depth: u8, threat: bool,
) -> Option<Vec<Pos>> {
    if depth > max_depth {
        return None;
    }

    for idx in 0 .. pos::BOARD_SIZE {
        let pattern = board.patterns.field[idx].clone();
        let (player_unit, opponent_unit) = match C {
            Color::Black => (pattern.black_unit, pattern.white_unit),
            Color::White => (pattern.white_unit, pattern.black_unit)
        };

        if !player_unit.has_four()
            || (C == Color::Black && pattern.is_forbidden())
            || (threat && opponent_unit.count_fives() != 1)
        {
            continue;
        }

        let four_pos = Pos::from_index(idx as u8);

        if player_unit.has_open_four() {
            return Some(vec![four_pos]);
        }

        board.set_mut(memo, four_pos);

        let defend_pos = find_defend_pos_unchecked::<C>(board);

        let defend_pattern = board.patterns.field[defend_pos.idx_usize()].clone();
        let defend_opponent_unit = match C {
            Color::Black => defend_pattern.white_unit,
            Color::White => defend_pattern.black_unit
        };

        board.set_mut(memo, defend_pos);

        let mut maybe_vcf = None;
        if !defend_opponent_unit.has_fours() && !defend_opponent_unit.has_open_four() {
            maybe_vcf =
                if (player_unit.has_three() && !defend_opponent_unit.has_four())
                    || (has_any_open_four::<C>(board) && !defend_opponent_unit.has_four())
                {
                    Some(vec![four_pos])
                } else {
                    try_vcf_v2::<C>(tt, memo, board, max_depth, depth + 2, defend_opponent_unit.has_four())
                }
        }

        board.unset_mut(memo, defend_pos);
        board.unset_mut(memo, four_pos);

        if let Some(mut vcf) = maybe_vcf {
            vcf.push(four_pos);
            return Some(vcf);
        }
    }

    None
}

fn find_defend_pos_unchecked<const C: Color>(board: &Board) -> Pos {
    let mut defend_pos = INVALID_POS;
    for defend_idx in 0 .. pos::BOARD_SIZE {
        if match C {
            Color::Black => board.patterns.field[defend_idx].black_unit,
            Color::White => board.patterns.field[defend_idx].white_unit
        }.has_five() {
            defend_pos = Pos::from_index(defend_idx as u8);
            break;
        }
    }

    defend_pos
}

fn has_any_open_four<const C: Color>(board: &Board) -> bool {
    for idx in 0 .. pos::BOARD_SIZE {
        if match C {
            Color::Black => board.patterns.field[idx].black_unit,
            Color::White => board.patterns.field[idx].white_unit
        }.has_open_four() {
            return true;
        }
    }

    false
}

// Depth-First Search(DFS)
fn try_vcf<const C: Color>(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board, max_depth: u8, depth: u8, three_opened: bool,
) -> Option<Vec<Pos>> {
    if depth > max_depth {
        return None;
    }

    for idx in 0 .. pos::BOARD_SIZE {
        let pattern = &board.patterns.field[idx];
        let player_unit = match C {
            Color::Black => pattern.black_unit,
            Color::White => pattern.white_unit
        };

        if !player_unit.has_four()
            || (C == Color::Black && pattern.is_forbidden())
        {
            continue;
        }

        let four_pos = Pos::from_index(idx as u8);

        if three_opened {
            if player_unit.has_open_four() {
                return Some(vec![four_pos]);
            } else {
                continue;
            }
        }

        if C == Color::White && player_unit.has_fours() {
            return Some(vec![four_pos])
        }

        board.set_mut(memo, four_pos);

        let maybe_vcf = defend_vcf::<C>(tt, memo, board, max_depth, depth + 1, player_unit.has_three());

        board.unset_mut(memo, four_pos);

        if let Some(mut vcf) = maybe_vcf {
            vcf.push(four_pos);
            return Some(vcf);
        }
    }

    None
}

fn defend_vcf<const C: Color>(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board, max_depth: u8, depth: u8, three_opened: bool,
) -> Option<Vec<Pos>> {
    let defend_pos = {
        let mut defend_pos = INVALID_POS;

        for idx in 0 .. pos::BOARD_SIZE {
            let pattern = &board.patterns.field[idx];
            let player_unit = match C {
                Color::Black => pattern.black_unit,
                Color::White => pattern.white_unit
            };

            if !player_unit.has_five() {
                continue;
            }

            if C == Color::White && pattern.is_forbidden() { // trap vcf
                return Some(vec![]);
           }
            
            if match C {
                Color::Black => pattern.white_unit,
                Color::White => pattern.black_unit
            }.has_four() { // opponent has a five
                if player_unit.has_four() {
                    defend_pos = INVALID_POS;
                    break;
                }

                return None;
            }
            
            defend_pos = Pos::from_index(idx as u8);
            break;
        }

        assert_ne!(defend_pos, INVALID_POS);
        defend_pos
    };

    board.set_mut(memo, defend_pos);

    let maybe_vcf = try_vcf::<C>(tt, memo, board, max_depth, depth + 1, three_opened);

    board.unset_mut(memo, defend_pos);

    maybe_vcf.map(|vcf| {
        let mut new_vcf = vcf;
        new_vcf.push(defend_pos);
        new_vcf
    })
}
