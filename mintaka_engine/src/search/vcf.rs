use crate::memo::transposition_table::TranspositionTable;
use mintaka::board::Board;
use mintaka::memo::slice_pattern_memo::SlicePatternMemo;
use mintaka::notation::color::Color;
use mintaka::notation::direction::Direction;
use mintaka::notation::pos;
use mintaka::notation::pos::{Pos, INVALID_POS};
use mintaka::pattern::PatternUnit;

pub fn vcf(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board, max_depth: u8
) -> Option<Vec<Pos>> {
    match board.player_color {
        Color::Black => vcf_offense::<{ Color::Black }>(tt, memo, board, max_depth, 0),
        Color::White => vcf_offense::<{ Color::White }>(tt, memo, board, max_depth, 0)
    }
}

// Depth-First Search(DFS)
fn vcf_offense<const C: Color>(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board, max_depth: u8, depth: u8,
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

        if player_unit.has_open_four()
            || (player_unit.has_three() && !opponent_has_open_four::<C>(board, player_unit, four_pos)) // three-four fork
            || (C == Color::White && player_unit.has_fours())
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

            if ! match C {
                Color::Black => pattern.black_unit,
                Color::White => pattern.white_unit
            }.has_five() {
                continue;
            }

            if C == Color::White && pattern.is_forbidden() { // trap vcf
                return Some(vec![]);
            }
            
            if match C {
                Color::Black => pattern.white_unit,
                Color::White => pattern.black_unit
            }.has_four() { // opponent has a five
                return None;
            }
            
            defend_pos = Pos::from_index(idx as u8);
            break;
        }

        assert_ne!(defend_pos, INVALID_POS);
        defend_pos
    };

    board.set_mut(memo, defend_pos);

    let maybe_vcf = vcf_offense::<C>(tt, memo, board, max_depth, depth + 1);

    board.unset_mut(memo, defend_pos);

    maybe_vcf.map(|vcf| {
        let mut new_vcf = vcf;
        new_vcf.push(defend_pos);
        new_vcf
    })
}

fn opponent_has_open_four<const C: Color>(board: &Board, player_unit: PatternUnit, four_pos: Pos) -> bool {
    let four_direction = player_unit.closed_four_direction_unchecked();

    fn open_four_at<const C: Color>(board: &Board, pos: Pos) -> bool {
        let pattern = board.patterns.field[pos.idx_usize()];

        match C {
            Color::Black => pattern.white_unit,
            Color::White => pattern.black_unit
        }.has_open_four()
    }

    open_four_at::<C>(board, four_pos.directional_offset_positive_unchecked(four_direction, 1))
        || open_four_at::<C>(board, four_pos.directional_offset_unchecked(four_direction, 1))
}

fn four_component(board: Board, direction: Direction, four_pos: Pos) -> Pos {
    match board.calculate_local_signature(direction, four_pos) {
        /* OOOV. */ 0b00011 => todo!(),
        /* OOO.V */ 0b00001 => todo!(),
        /* OOV.O */ 0b10011 => todo!(),
        /* OO.VO */ 0b01001 => todo!(),
        /* OV.OO */ 0b10010 => todo!(),
        /* O.VOO */ 0b11001 => todo!(),
        /* V.OOO */ 0b10000 => todo!(),
        /* .VOOO */ 0b11000 => todo!(),
        /* VOO.O */ 0b11000 => todo!(),
        /* .OOVO */ 0b01011 => todo!(),
        /* OO.OV */ 0b00010 => todo!(),
        /* VO.OO */ 0b01000 => todo!(),
        /* OVOO. */ 0b11010 => todo!(),
        /* O.OOV */ 0b00011 => todo!(),
        _ => unreachable!()
    }
}
