use crate::memo::transposition_table::TranspositionTable;
use crate::memo::tt_entry::{EndgameFlag, TTEntry, TTFlag};
use crate::value::{Eval, Score};
use rusty_renju::board::Board;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use rusty_renju::pattern::PatternCount;

pub fn vcf(
    tt: &mut TranspositionTable, board: &mut Board, max_depth: u8
) -> Score {
    todo!()
}

pub fn vcf_sequence(
    tt: &mut TranspositionTable, board: &mut Board, max_depth: u8
) -> Option<Vec<Pos>> {
    match board.player_color {
        Color::Black => try_vcf::<{ Color::Black }>(tt, board, max_depth, 0, false),
        Color::White => try_vcf::<{ Color::White }>(tt, board, max_depth, 0, false),
    }.map(|mut result| {
        result.reverse();
        result
    })
}

// Depth-First Search(DFS)
pub fn try_vcf<const C: Color>(
    tt: &mut TranspositionTable, board: &mut Board,
    max_depth: u8, depth: u8, opponent_has_five: bool,
) -> Option<Vec<Pos>> {
    if depth > max_depth || board.stones > pos::U8_BOARD_SIZE - 2 {
        return None;
    }

    for idx in 0 .. pos::BOARD_SIZE {
        let pattern = board.patterns.field[idx];
        let player_unit = pattern.player_unit::<C>();
        let opponent_unit = pattern.opponent_unit::<C>();

        if !player_unit.has_four()
            || (opponent_has_five && !opponent_unit.has_five())
            || (C == Color::Black && pattern.is_forbidden())
        {
            continue;
        }

        let four_pos = Pos::from_index(idx as u8);

        if player_unit.has_open_four() {
            tt.store_mut(board.hash_key, build_vcf_win_tt_entry(0, four_pos));

            return Some(vec![four_pos]);
        }

        board.set_mut(four_pos);

        let defend_pos = find_defend_pos_unchecked::<C>(board);
        let defend_pattern = board.patterns.field[defend_pos.idx_usize()];
        let defend_unit = defend_pattern.opponent_unit::<C>();
        let defend_four_count = defend_unit.count_fours();
        let defend_is_forbidden = C == Color::White && defend_pattern.is_forbidden();

        let maybe_vcf =
        if match C {
            Color::Black => defend_four_count != PatternCount::Multiple
                && !defend_unit.has_open_four(),
            Color::White => !defend_unit.has_open_four()
                || defend_is_forbidden
        } {
            if match C {
                Color::Black => defend_four_count == PatternCount::Cold
                    && player_unit.has_three(),
                Color::White => defend_is_forbidden
                    || (defend_four_count == PatternCount::Cold
                     && player_unit.has_three())
            } {
                tt.store_mut(board.hash_key, build_vcf_win_tt_entry(0, four_pos));

                Some(vec![four_pos])
            } else if !tt.probe(
                board.hash_key.set(C.reversed(), defend_pos)
            ).is_some_and(|entry| entry.endgame_flag == EndgameFlag::Cold) {
                board.set_mut(defend_pos);

                let maybe_vcf = try_vcf::<C>(tt, board, max_depth, depth + 2, defend_four_count != PatternCount::Cold)
                    .map(|mut vcf| {
                        vcf.push(defend_pos);
                        vcf.push(four_pos);
                        vcf
                    });

                board.unset_mut(defend_pos);

                maybe_vcf
            } else {
                None
            }
        } else {
            None
        };

        board.unset_mut(four_pos);

        if maybe_vcf.is_some() {
            return maybe_vcf;
        }
    }

    let tt_entry = tt.probe(board.hash_key)
        .map(|mut tt_entry| {
            tt_entry.endgame_flag = EndgameFlag::Cold;
            tt_entry
        })
        .unwrap_or_else(|| TTEntry {
            best_move: Pos::INVALID,
            depth,
            flag: Default::default(),
            endgame_flag: EndgameFlag::Cold,
            score: 0,
            eval: 0,
        });

    tt.store_mut(board.hash_key, tt_entry);

    None
}

fn find_defend_pos_unchecked<const C: Color>(board: &Board) -> Pos {
    let mut defend_pos = Pos::INVALID;
    for defend_idx in 0 .. pos::BOARD_SIZE {
        if board.patterns.field[defend_idx].player_unit::<C>().has_five() {
            defend_pos = Pos::from_index(defend_idx as u8);
            break;
        }
    }

    defend_pos
}

fn build_vcf_win_tt_entry(depth: u8, four_pos: Pos) -> TTEntry {
    TTEntry {
        best_move: four_pos,
        depth,
        flag: TTFlag::Exact,
        endgame_flag: EndgameFlag::Win,
        score: Score::MAX,
        eval: Eval::MAX,
    }
}
