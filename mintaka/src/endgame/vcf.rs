use crate::endgame::accumulator::{EndgameAccumulator, SequenceEndgameAccumulator};
use crate::memo::tt_entry::{EndgameFlag, ScoreKind, TTEntry, TTFlag};
use crate::thread_data::ThreadData;
use rusty_renju::board::Board;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::value::{Depth, Eval, Score};
use rusty_renju::pattern::PatternCount;

#[derive(Copy, Clone)]
#[repr(align(8))]
pub struct VCFFrame {
    board: Board,
    next_idx: usize,
    depth: Depth,
    opponent_has_five: bool,
    four_pos: Pos,
    defend_pos: Pos,
}

pub fn vcf_search(
    td: &mut ThreadData,
    board: &Board, max_depth: Depth,
) -> Score {
    vcf::<Score>(td, board, max_depth)
}

pub fn vcf_sequence(
    td: &mut ThreadData,
    board: &Board, max_depth: Depth
) -> Option<Vec<Pos>> {
    vcf::<SequenceEndgameAccumulator>(td, board, max_depth)
        .map(|mut sequence| {
            sequence.reverse();
            sequence
        })
}

fn vcf<ACC: EndgameAccumulator>(
    td: &mut ThreadData,
    board: &Board, max_depth: Depth
) -> ACC {
    let board = *board;
    match board.player_color {
        Color::Black => try_vcf::<{ Color::Black }, ACC>(td, board, max_depth, 0, false),
        Color::White => try_vcf::<{ Color::White }, ACC>(td, board, max_depth, 0, false),
    }
}

// depth-first search
fn try_vcf<const C: Color, ACC: EndgameAccumulator>(
    td: &mut ThreadData,
    mut board: Board,
    max_depth: Depth, mut vcf_ply: Depth, mut opponent_has_five: bool,
) -> ACC {
    let mut idx: usize = 0;

    #[inline]
    fn backtrace_frames<ACC: EndgameAccumulator>(
        td: &mut ThreadData,
        board: Board, depth: Depth, four_pos: Pos
    ) -> ACC {
        let mut result = ACC::unit(four_pos);
        let mut hash_key = board.hash_key;

        let opponent_color = board.opponent_color();

        while let Some(frame) = td.vcf_stack.pop() {
            hash_key = hash_key.set(opponent_color, frame.defend_pos);
            td.tt.store_entry_mut(hash_key, build_vcf_lose_tt_entry(depth));

            hash_key = hash_key.set(board.player_color, frame.four_pos);
            td.tt.store_entry_mut(hash_key, build_vcf_win_tt_entry(depth, frame.four_pos));

            result = result.append(frame.defend_pos, frame.four_pos);
        }

        td.batch_counter.add_single_mut();

        result
    }

    'vcf_search: loop {
        'position_search: while idx < pos::BOARD_SIZE {
            let pattern = board.patterns.field[idx];
            let player_unit = pattern.player_unit::<C>();
            let opponent_unit = pattern.opponent_unit::<C>();

            if !player_unit.has_any_four()
                || (opponent_has_five && !opponent_unit.has_five())
                || (C == Color::Black && pattern.is_forbidden())
            {
                idx += 1;

                continue 'position_search;
            }

            let four_pos = Pos::from_index(idx as u8);

            if player_unit.has_open_four() {
                td.tt.store_entry_mut(board.hash_key, build_vcf_win_tt_entry(vcf_ply, four_pos));

                return backtrace_frames(td, board, vcf_ply, four_pos);
            }

            let mut position_board = board.set(four_pos);
            td.batch_counter.add_single_mut();

            let defend_pos = position_board.patterns.unchecked_five_pos.player_unit::<C>().unwrap();
            let defend_pattern = position_board.patterns.field[defend_pos.idx_usize()];
            let defend_unit = defend_pattern.opponent_unit::<C>();
            let defend_four_count = defend_unit.count_fours();
            let defend_is_forbidden = C == Color::White && defend_pattern.is_forbidden();

            if match C {
                Color::Black => defend_four_count == PatternCount::Multiple
                    || defend_unit.has_open_four(),
                Color::White => defend_unit.has_open_four()
                    && !defend_is_forbidden
            } {
                idx += 1;
                continue 'position_search;
            }

            if match C {
                Color::Black => defend_four_count == PatternCount::Cold
                    && player_unit.has_three(),
                Color::White => defend_is_forbidden
                    || (defend_four_count == PatternCount::Cold
                    && player_unit.has_three())
            } {
                td.tt.store_entry_mut(board.hash_key, build_vcf_win_tt_entry(vcf_ply, four_pos));

                return backtrace_frames(td, board, vcf_ply, four_pos);
            }

            if position_board.stones + 3 >= pos::U8_BOARD_SIZE || vcf_ply + 4 > max_depth
                || td.tt.probe(position_board.hash_key.set(C.reversed(), defend_pos))
                .is_some_and(|entry| entry.tt_flag.endgame_flag() == EndgameFlag::Cold)
            {
                idx += 1;
                continue 'position_search;
            }

            position_board.set_mut(defend_pos);
            td.batch_counter.add_single_mut();

            td.vcf_stack.push(VCFFrame {
                board,
                next_idx: idx + 1,
                depth: vcf_ply,
                opponent_has_five,
                four_pos,
                defend_pos,
            });

            board = position_board;
            idx = 0;
            vcf_ply = vcf_ply + 2;
            opponent_has_five = defend_four_count != PatternCount::Cold;

            continue 'vcf_search;
        }

        let tt_entry = td.tt.probe(board.hash_key)
            .map(|mut tt_entry| {
                tt_entry.tt_flag.set_endgame_flag(EndgameFlag::Cold);
                tt_entry
            })
            .unwrap_or_else(|| TTEntry {
                best_move: Pos::INVALID,
                depth: vcf_ply,
                age: td.tt.fetch_age(),
                tt_flag: TTFlag::new(ScoreKind::None, EndgameFlag::Cold, false),
                score: 0,
                eval: 0,
            });

        td.tt.store_entry_mut(board.hash_key, tt_entry);

        if let Some(frame) = td.vcf_stack.pop() {
            board = frame.board;
            idx = frame.next_idx;
            vcf_ply = frame.depth;
            opponent_has_five = frame.opponent_has_five;
        } else {
            break 'vcf_search;
        }
    }

    ACC::COLD
}

#[inline]
fn build_vcf_win_tt_entry(depth: Depth, four_pos: Pos) -> TTEntry {
    TTEntry {
        best_move: four_pos,
        depth,
        age: u8::MAX,
        tt_flag: TTFlag::new(
            ScoreKind::Exact,
            EndgameFlag::Win,
            false,
        ),
        score: Score::MAX,
        eval: Eval::MAX,
    }
}

#[inline]
fn build_vcf_lose_tt_entry(depth: Depth) -> TTEntry {
   TTEntry {
       best_move: Pos::INVALID,
       depth,
       age: u8::MAX,
       tt_flag: TTFlag::new(
           ScoreKind::Exact,
           EndgameFlag::Lose,
           false,
       ),
       score: Score::MIN,
       eval: Eval::MIN,
   }
}
