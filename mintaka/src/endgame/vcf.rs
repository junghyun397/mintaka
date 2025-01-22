use crate::memo::transposition_table::TranspositionTable;
use crate::memo::tt_entry::{EndgameFlag, TTEntry, TTFlag};
use crate::value::{Eval, Score};
use rusty_renju::board::Board;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use rusty_renju::pattern::PatternCount;

trait VCFAccumulator {

    const COLD: Self;

    fn unit(pos: Pos) -> Self;

    fn append(self, defend: Pos, four: Pos) -> Self;

    fn has_result(&self) -> bool;

}

type SequenceVCFAccumulator = Option<Vec<Pos>>;

impl VCFAccumulator for SequenceVCFAccumulator {

    const COLD: Self = None;

    fn unit(pos: Pos) -> Self {
        Some(vec![pos])
    }

    fn append(self, defend: Pos, four: Pos) -> Self {
        self.map(|mut sequence| {
            sequence.push(defend);
            sequence.push(four);
            sequence
        })
    }

    fn has_result(&self) -> bool {
        self.is_some()
    }

}

impl VCFAccumulator for Score {

    const COLD: Self = 0;

    fn unit(_pos: Pos) -> Self {
        Score::MAX
    }

    fn append(self, _defend: Pos, _four: Pos) -> Self {
        self
    }

    fn has_result(&self) -> bool {
        *self != 0
    }

}

pub fn vcf<Acc: VCFAccumulator>(
    tt: &mut TranspositionTable, board: &mut Board, max_depth: u8
) -> Acc {
    match board.player_color {
        Color::Black => try_vcf::<{ Color::Black }, Acc>(tt, board, max_depth, 0, false),
        Color::White => try_vcf::<{ Color::White }, Acc>(tt, board, max_depth, 0, false),
    }
}

pub fn vcf_sequence(
    tt: &mut TranspositionTable, board: &mut Board, max_depth: u8
) -> Option<Vec<Pos>> {
    vcf::<SequenceVCFAccumulator>(tt, board, max_depth)
        .map(|mut sequence| {
            sequence.reverse();
            sequence
        })
}

// Depth-First Search(DFS)
pub fn try_vcf<const C: Color, ACC: VCFAccumulator>(
    tt: &mut TranspositionTable, board: &mut Board,
    max_depth: u8, depth: u8, opponent_has_five: bool,
) -> ACC {
    if depth > max_depth || board.stones > pos::U8_BOARD_SIZE - 2 {
        return ACC::COLD;
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
            tt.store_mut(board.hash_key, build_vcf_win_tt_entry(depth, four_pos));

            return ACC::unit(four_pos);
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
                tt.store_mut(board.hash_key, build_vcf_win_tt_entry(depth + 1, four_pos));

                ACC::unit(four_pos)
            } else if !tt.probe(
                board.hash_key.set(C.reversed(), defend_pos)
            ).is_some_and(|entry| entry.endgame_flag == EndgameFlag::Cold) {
                board.set_mut(defend_pos);

                let maybe_vcf = try_vcf::<C, ACC>(tt, board, max_depth, depth + 2, defend_four_count != PatternCount::Cold)
                    .append(defend_pos, four_pos);

                board.unset_mut(defend_pos);

                maybe_vcf
            } else {
                ACC::COLD
            }
        } else {
            ACC::COLD
        };

        board.unset_mut(four_pos);

        if maybe_vcf.has_result() {
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
            flag: TTFlag::default(),
            endgame_flag: EndgameFlag::Cold,
            score: 0,
            eval: 0,
        });

    tt.store_mut(board.hash_key, tt_entry);

    ACC::COLD
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
