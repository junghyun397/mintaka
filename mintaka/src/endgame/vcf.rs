use crate::endgame::accumulator::{EndgameAccumulator, SequenceEndgameAccumulator};
use crate::memo::tt_entry::{EndgameFlag, ScoreKind, TTEntry, TTFlag};
use crate::movegen::move_generator::{generate_vcf_moves, VcfMoves};
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use rusty_renju::board::Board;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::value::{Depth, Eval, Score};
use rusty_renju::pattern::{Pattern, PatternCount};

pub trait VcfDestination {

    fn additional_abort(&self, defend_pattern: Pattern) -> bool;

    fn additional_reached(&self, four_pos: Pos) -> bool;

}

pub struct VcfWin; impl VcfDestination for VcfWin {
    fn additional_abort(&self, _defend_pattern: Pattern) -> bool {
        false
    }

    fn additional_reached(&self, _four_pos: Pos) -> bool {
        false
    }
}

pub struct VcfDefend {
    target_pos: Pos
}

impl VcfDestination for VcfDefend {
    fn additional_abort(&self, defend_pattern: Pattern) -> bool {
        defend_pattern.has_three()
    }

    fn additional_reached(&self, four_pos: Pos) -> bool {
        self.target_pos == four_pos
    }
}

#[derive(Copy, Clone)]
pub struct VcfFrame {
    board: Board,
    vcf_moves: VcfMoves,
    next_move_counter: usize,
    depth: Depth,
    four_pos: Pos,
    defend_pos: Pos,
}

pub fn vcf_search(
    td: &mut ThreadData<impl ThreadType>,
    board: &Board, max_depth: Depth,
) -> Score {
    let vcf_moves = generate_vcf_moves(&board, board.player_color, Score::DISTANCE_WINDOW, pos::CENTER);

    vcf::<Score>(td, VcfWin, board, vcf_moves, max_depth)
}

pub fn vcf_defend(
    td: &mut ThreadData<impl ThreadType>,
    board: &Board, max_depth: Depth,
    target_pos: Pos
) -> Score {
    let vcf_moves = generate_vcf_moves(&board, board.player_color, 8, pos::CENTER);

    vcf::<Score>(td, VcfDefend { target_pos }, board, vcf_moves, max_depth)
}

pub fn vcf_sequence(
    td: &mut ThreadData<impl ThreadType>,
    board: &Board, max_depth: Depth
) -> Option<Vec<Pos>> {
    let vcf_moves = generate_vcf_moves(&board, board.player_color, 8, pos::CENTER);

    vcf::<SequenceEndgameAccumulator>(td, VcfWin, board, vcf_moves, max_depth)
        .map(|mut sequence| {
            sequence.reverse();
            sequence
        })
}

fn vcf<ACC: EndgameAccumulator>(
    td: &mut ThreadData<impl ThreadType>, dest: impl VcfDestination,
    board: &Board, vcf_moves: VcfMoves, max_depth: Depth
) -> ACC {
    let board = *board;
    match board.player_color {
        Color::Black => try_vcf::<{ Color::Black }, ACC>(td, dest, board, vcf_moves, max_depth, 0),
        Color::White => try_vcf::<{ Color::White }, ACC>(td, dest, board, vcf_moves, max_depth, 0),
    }
}

// depth-first search
fn try_vcf<const C: Color, ACC: EndgameAccumulator>(
    td: &mut ThreadData<impl ThreadType>, dest: impl VcfDestination,
    mut board: Board, mut vcf_moves: VcfMoves,
    max_depth: Depth, mut vcf_ply: Depth,
) -> ACC {
    let mut move_counter: usize = 0;

    #[inline]
    fn backtrace_frames<ACC: EndgameAccumulator>(
        td: &mut ThreadData<impl ThreadType>,
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
        'position_search: for (seq, four_pos) in vcf_moves.moves.into_iter()
            .take(vcf_moves.len as usize)
            .skip(move_counter)
            .enumerate()
        {
            let idx = four_pos.idx_usize();

            let player_pattern = board.patterns.field.player_unit::<C>()[idx];

            if C == Color::Black && player_pattern.is_forbidden() {
                continue 'position_search;
            }

            if player_pattern.has_open_four() {
                td.tt.store_entry_mut(board.hash_key, build_vcf_win_tt_entry(vcf_ply, four_pos));

                return backtrace_frames(td, board, vcf_ply, four_pos);
            }

            let mut position_board = board.set(four_pos);
            td.batch_counter.add_single_mut();

            let defend_pos = position_board.patterns.unchecked_five_pos.player_unit::<C>().unwrap();
            let defend_pattern = position_board.patterns.field.opponent_unit::<C>()[defend_pos.idx_usize()];
            let defend_four_count = defend_pattern.count_fours();
            let defend_is_forbidden = C == Color::White && defend_pattern.is_forbidden();

            if match C {
                Color::Black => defend_four_count == PatternCount::Multiple
                    || defend_pattern.has_open_four(),
                Color::White => defend_pattern.has_open_four()
                    && !defend_is_forbidden
            } || dest.additional_abort(defend_pattern) {
                continue 'position_search;
            }

            if (C == Color::White && defend_is_forbidden) || (
                defend_four_count == PatternCount::Cold
                    && (player_pattern.has_three() || dest.additional_reached(four_pos))
            ) {
                td.tt.store_entry_mut(board.hash_key, build_vcf_win_tt_entry(vcf_ply, four_pos));

                return backtrace_frames(td, board, vcf_ply, four_pos);
            }

            if position_board.stones + 3 >= pos::U8_BOARD_SIZE || vcf_ply + 4 > max_depth
                || td.tt.probe(position_board.hash_key.set(C.reversed(), defend_pos))
                .is_some_and(|entry| entry.tt_flag.endgame_flag() == EndgameFlag::Cold)
            {
                continue 'position_search;
            }

            position_board.set_mut(defend_pos);
            td.batch_counter.add_single_mut();

            td.vcf_stack.push(VcfFrame {
                board,
                vcf_moves,
                next_move_counter: seq + 1,
                depth: vcf_ply,
                four_pos,
                defend_pos,
            });

            vcf_moves = if defend_four_count != PatternCount::Cold {
                let defend_move = position_board.patterns.unchecked_five_pos.opponent_unit::<C>().unwrap();

                if !position_board.patterns.field.player_unit::<C>()[defend_move.idx_usize()].has_any_four() {
                    board = position_board;
                    vcf_ply = vcf_ply + 2;
                    break 'position_search;
                }

                let mut moves = [Pos::INVALID; 31];
                moves[0] = position_board.patterns.unchecked_five_pos.opponent_unit::<C>().unwrap();
                VcfMoves { moves, len: 1 }
            } else {
                generate_vcf_moves(&position_board, C, ACC::DISTANCE_WINDOW, defend_pos)
            };

            move_counter = 0;
            board = position_board;
            vcf_ply = vcf_ply + 2;

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
                age: td.tt.age,
                tt_flag: TTFlag::new(ScoreKind::None, EndgameFlag::Cold, false),
                score: 0,
                eval: 0,
            });

        td.tt.store_entry_mut(board.hash_key, tt_entry);

        if let Some(frame) = td.vcf_stack.pop() {
            board = frame.board;
            vcf_moves = frame.vcf_moves;
            move_counter = frame.next_move_counter;
            vcf_ply = frame.depth;
        } else {
            break 'vcf_search;
        }
    }

    ACC::ZERO
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
