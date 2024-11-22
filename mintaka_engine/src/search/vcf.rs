use crate::memo::transposition_table::TranspositionTable;
use crate::memo::tt_entry::Score;
use mintaka::board::Board;
use mintaka::memo::slice_pattern_memo::SlicePatternMemo;
use mintaka::notation::color::Color;
use mintaka::notation::pos;
use mintaka::notation::pos::{Pos, INVALID_POS, U8_BOARD_SIZE};
use mintaka::pattern::PatternCount;

pub fn vcf(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board, max_depth: u8
) -> Score {
    todo!()
}

pub fn vcf_sequence(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board, max_depth: u8
) -> Option<Vec<Pos>> {
    match board.player_color {
        Color::Black => try_vcf::<{ Color::Black }>(tt, memo, board, max_depth, 0, opponent_has_any_five::<{ Color::Black }>(board)),
        Color::White => try_vcf::<{ Color::White }>(tt, memo, board, max_depth, 0, opponent_has_any_five::<{ Color::Black }>(board)),
    }.map(|mut result| {
        result.reverse();
        result
    })
}

// Depth-First Search(DFS)
fn try_vcf<const C: Color>(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board, max_depth: u8, depth: u8, opponent_has_five: bool,
) -> Option<Vec<Pos>> {
    if board.stones > U8_BOARD_SIZE - 2 {
        return None;
    }

    for idx in 0 .. pos::BOARD_SIZE {
        let pattern = board.patterns.field[idx].clone();
        let player_unit = pattern.player_unit::<C>();
        let opponent_unit = pattern.opponent_unit::<C>();

        if !player_unit.has_four()
            || (C == Color::Black && pattern.is_forbidden())
            || (opponent_has_five && !opponent_unit.has_five())
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
        let defend_opponent_unit = defend_pattern.opponent_unit::<C>();
        let opponent_four_count = defend_opponent_unit.count_fours();

        board.set_mut(memo, defend_pos);

        let maybe_vcf =
        if opponent_four_count != PatternCount::Multiple
            && !defend_opponent_unit.has_open_four()
        {
            if opponent_four_count == PatternCount::Cold
                && (player_unit.has_three() || player_has_any_open_four::<C>(board))
            {
                Some(vec![four_pos])
            } else {
                try_vcf::<C>(tt, memo, board, max_depth, depth + 2, opponent_four_count != PatternCount::Cold)
                    .map(|mut vcf| {
                        vcf.push(defend_pos);
                        vcf.push(four_pos);
                        vcf
                    })
            }
        } else {
            None
        };

        board.unset_mut(memo, defend_pos);
        board.unset_mut(memo, four_pos);

        if maybe_vcf.is_some() {
            return maybe_vcf;
        }
    }

    None
}

fn find_defend_pos_unchecked<const C: Color>(board: &Board) -> Pos {
    let mut defend_pos = INVALID_POS;
    for defend_idx in 0 .. pos::BOARD_SIZE {
        if board.patterns.field[defend_idx].player_unit::<C>().has_five() {
            defend_pos = Pos::from_index(defend_idx as u8);
            break;
        }
    }

    defend_pos
}

fn player_has_any_open_four<const C: Color>(board: &Board) -> bool {
    for idx in 0 .. pos::BOARD_SIZE {
        if board.patterns.field[idx].player_unit::<C>().has_open_four() {
            return true;
        }
    }

    false
}

fn opponent_has_any_five<const C: Color>(board: &Board) -> bool {
    for idx in 0 .. pos::BOARD_SIZE {
        if board.patterns.field[idx].opponent_unit::<C>().has_five() {
            return true;
        }
    }

    false
}

// Depth-First Search(DFS)
fn try_vcf_legacy<const C: Color>(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board, max_depth: u8, depth: u8
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

        if C == Color::White && player_unit.has_fours() {
            return Some(vec![four_pos])
        }

        board.set_mut(memo, four_pos);

        let maybe_vcf = defend_vcf::<C>(tt, memo, board, max_depth, depth + 1);

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
    board: &mut Board, max_depth: u8, depth: u8
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

    let maybe_vcf = try_vcf_legacy::<C>(tt, memo, board, max_depth, depth + 1);

    board.unset_mut(memo, defend_pos);

    maybe_vcf.map(|vcf| {
        let mut new_vcf = vcf;
        new_vcf.push(defend_pos);
        new_vcf
    })
}
