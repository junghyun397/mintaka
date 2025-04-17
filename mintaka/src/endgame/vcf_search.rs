use crate::endgame::accumulator::{EndgameAccumulator, SequenceEndgameAccumulator};
use crate::game_state::GameState;
use crate::memo::tt_entry::{EndgameFlag, ScoreKind, TTEntry, TTFlag};
use crate::movegen::move_generator::{generate_vcf_moves, VcfMovesUnchecked};
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use rusty_renju::board::Board;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::value::{Depth, Score, Scores};
use rusty_renju::pattern::{Pattern, PatternCount};

pub trait VcfDestination {

    fn conditional_abort(&self, defend_pattern: Pattern) -> bool;

    fn additional_reached(&self, four_pos: Pos) -> bool;

}

pub struct VcfWin; impl VcfDestination for VcfWin {
    fn conditional_abort(&self, _defend_pattern: Pattern) -> bool {
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
    fn conditional_abort(&self, defend_pattern: Pattern) -> bool {
        defend_pattern.has_three()
    }

    fn additional_reached(&self, four_pos: Pos) -> bool {
        self.target_pos == four_pos
    }
}

#[derive(Copy, Clone)]
pub struct VcfFrame {
    vcf_moves: VcfMovesUnchecked,
    next_move_counter: usize,
    depth: Depth,
    four_pos: Pos,
    defend_pos: Pos,
}

pub fn vcf_search(
    td: &mut ThreadData<impl ThreadType>,
    state: &GameState, max_depth: Depth,
) -> Option<Score> {
    let mut vcf_moves = generate_vcf_moves(
        &state.board,
        state.board.player_color,
        Score::DISTANCE_WINDOW,
        state.history.recent_player_move_unchecked()
    );

    (vcf_moves.top != 0).then(|| {
        vcf_moves.sort_moves(state.history.recent_player_move_unchecked());

        vcf::<Score>(td, VcfWin, state.board, vcf_moves, max_depth)
    })
}

pub fn vcf_defend(
    td: &mut ThreadData<impl ThreadType>,
    state: &GameState, max_depth: Depth,
    target_pos: Pos
) -> Score {
    let vcf_moves = generate_vcf_moves(
        &state.board,
        state.board.player_color,
        8,
        state.history.recent_opponent_move_unchecked()
    );

    vcf::<Score>(td, VcfDefend { target_pos }, state.board, vcf_moves, max_depth)
}

pub fn vcf_sequence(
    td: &mut ThreadData<impl ThreadType>,
    board: &Board, max_depth: Depth
) -> Option<Vec<Pos>> {
    let vcf_moves = generate_vcf_moves(board, board.player_color, 8, pos::CENTER);

    vcf::<SequenceEndgameAccumulator>(td, VcfWin, *board, vcf_moves, max_depth)
        .map(|mut sequence| {
            sequence.reverse();
            sequence
        })
}

fn vcf<ACC: EndgameAccumulator>(
    td: &mut ThreadData<impl ThreadType>, dest: impl VcfDestination,
    board: Board, vcf_moves: VcfMovesUnchecked, max_depth: Depth
) -> ACC {
    match board.player_color {
        Color::Black => try_vcf::<{ Color::Black }, ACC>(td, dest, board, vcf_moves, max_depth, 0),
        Color::White => try_vcf::<{ Color::White }, ACC>(td, dest, board, vcf_moves, max_depth, 0),
    }
}

// depth-first search
fn try_vcf<const C: Color, ACC: EndgameAccumulator>(
    td: &mut ThreadData<impl ThreadType>, dest: impl VcfDestination,
    mut board: Board, mut vcf_moves: VcfMovesUnchecked,
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

            result = result.append_pos(frame.defend_pos, frame.four_pos);
        }

        result
    }

    'vcf_search: loop {
        'position_search: for (seq, four_pos) in vcf_moves.moves.into_iter()
            .enumerate()
            .take(vcf_moves.top as usize)
            .skip(move_counter)
        {
            let idx = four_pos.idx_usize();

            let player_pattern = board.patterns.field.get_ref::<C>()[idx];

            if C == Color::Black && player_pattern.is_forbidden() {
                continue 'position_search;
            }

            if player_pattern.has_open_four() {
                td.tt.store_entry_mut(board.hash_key, build_vcf_win_tt_entry(vcf_ply, four_pos));

                return backtrace_frames(td, board, vcf_ply, four_pos);
            }

            board.set_mut(four_pos);
            td.batch_counter.add_single_mut();

            let defend_pos = board.patterns.unchecked_five_pos.get_ref::<C>().unwrap();
            let tt_key = board.hash_key.set(C.reversed(), defend_pos);
            td.tt.prefetch(tt_key);

            let defend_pattern = board.patterns.field.get_reversed::<C>()[defend_pos.idx_usize()];
            let defend_four_count = defend_pattern.count_fours();
            let defend_is_forbidden = C == Color::White && defend_pattern.is_forbidden();

            if match C {
                Color::Black => defend_four_count == PatternCount::Multiple
                    || defend_pattern.has_open_four(),
                Color::White => defend_pattern.has_open_four()
                    && !defend_is_forbidden
            } || dest.conditional_abort(defend_pattern) {
                board.unset_mut(four_pos);
                continue 'position_search;
            }

            if (C == Color::White && defend_is_forbidden) || (
                defend_four_count == PatternCount::Cold
                    && (player_pattern.has_three() || dest.additional_reached(four_pos))
            ) {
                td.tt.store_entry_mut(board.hash_key, build_vcf_win_tt_entry(vcf_ply, four_pos));

                return backtrace_frames(td, board, vcf_ply, four_pos);
            }

            if board.stones + 3 >= pos::U8_BOARD_SIZE || vcf_ply + 4 > max_depth
                || td.tt.probe(tt_key)
                .is_some_and(|entry| entry.tt_flag.endgame_flag() == EndgameFlag::Cold)
            {
                board.unset_mut(four_pos);
                continue 'position_search;
            }

            board.set_mut(defend_pos);
            td.batch_counter.add_single_mut();

            td.vcf_stack.push(VcfFrame {
                vcf_moves,
                next_move_counter: seq + 1,
                depth: vcf_ply,
                four_pos,
                defend_pos,
            });

            let next_vcf_moves = if defend_four_count != PatternCount::Cold {
                let defend_move = board.patterns.unchecked_five_pos.get_reversed_ref::<C>().unwrap();

                if !board.patterns.field.get_ref::<C>()[defend_move.idx_usize()].has_any_four() {
                    vcf_ply += 2;
                    break 'position_search;
                }

                let mut moves = [MaybePos::NONE.unwrap(); 31];
                moves[0] = board.patterns.unchecked_five_pos.get_reversed_ref::<C>().unwrap();
                VcfMovesUnchecked { moves, top: 1 }
            } else {
                generate_vcf_moves(&board, C, ACC::DISTANCE_WINDOW, four_pos)
            };

            vcf_moves = next_vcf_moves;
            move_counter = 0;
            vcf_ply += 2;

            continue 'vcf_search;
        }

        let tt_entry = td.tt.probe(board.hash_key)
            .map(|mut tt_entry| {
                tt_entry.tt_flag.set_endgame_flag(EndgameFlag::Cold);
                tt_entry
            })
            .unwrap_or_else(|| TTEntry {
                best_move: MaybePos::NONE,
                depth: vcf_ply,
                age: td.tt.age,
                tt_flag: TTFlag::new(ScoreKind::Exact, EndgameFlag::Cold, false),
                score: 0,
                eval: 0,
            });

        td.tt.store_entry_mut(board.hash_key, tt_entry);

        if let Some(frame) = td.vcf_stack.pop() {
            board.unset_mut(frame.defend_pos);
            board.unset_mut(frame.four_pos);

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
        best_move: MaybePos::new(four_pos),
        depth,
        age: u8::MAX,
        tt_flag: TTFlag::new(
            ScoreKind::Exact,
            EndgameFlag::Win,
            false,
        ),
        score: Score::INF,
        eval: Score::INF,
    }
}

#[inline]
fn build_vcf_lose_tt_entry(depth: Depth) -> TTEntry {
   TTEntry {
       best_move: MaybePos::NONE,
       depth,
       age: u8::MAX,
       tt_flag: TTFlag::new(
           ScoreKind::Exact,
           EndgameFlag::Lose,
           false,
       ),
       score: -Score::INF,
       eval: -Score::INF,
   }
}
