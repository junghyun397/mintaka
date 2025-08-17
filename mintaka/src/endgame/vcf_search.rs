use crate::endgame::accumulator::{EndgameAccumulator, SequenceEndgameAccumulator};
use crate::eval::evaluator::Evaluator;
use crate::game_state::GameState;
use crate::memo::transposition_table::TTView;
use crate::memo::tt_entry::{EndgameFlag, ScoreKind, TTEntry, TTFlag};
use crate::movegen::move_generator::{generate_vcf_moves, VcfMovesUnchecked};
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use rusty_renju::board::Board;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::value::{Score, Scores};
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
    four_pos: Pos,
    defend_pos: Pos,
}

pub fn vcf_search(
    td: &mut ThreadData<impl ThreadType, impl Evaluator>,
    max_vcf_ply: usize,
    state: &GameState,
    alpha: Score,
    beta: Score,
) -> Option<Score> {
    if state.board.patterns.counts.global.access(state.board.player_color).total_fours() == 0 {
        return None;
    }

    let mut vcf_moves = generate_vcf_moves(
        &state.board,
        Score::DISTANCE_WINDOW,
        state.history.recent_player_action().unwrap_or(pos::CENTER)
    );

    if vcf_moves.is_empty() {
        return None;
    }

    vcf_moves.sort_moves(state.history.recent_player_action().unwrap_or(pos::CENTER));

    Some(vcf::<Score>(td, VcfWin, max_vcf_ply, *state, vcf_moves, alpha, beta))
}

pub fn vcf_defend(
    td: &mut ThreadData<impl ThreadType, impl Evaluator>,
    max_vcf_ply: usize,
    state: &GameState,
    target_pos: Pos
) -> Score {
    let vcf_moves = generate_vcf_moves(
        &state.board,
        8,
        state.history.recent_action().unwrap_or(target_pos)
    );

    vcf::<Score>(td, VcfDefend { target_pos }, max_vcf_ply, *state, vcf_moves, Score::MIN, Score::MAX)
}

pub fn vcf_sequence(
    td: &mut ThreadData<impl ThreadType, impl Evaluator>,
    state: &GameState
) -> Option<Vec<MaybePos>> {
    let vcf_moves = generate_vcf_moves(&state.board, 8, pos::CENTER);

    vcf::<SequenceEndgameAccumulator>(td, VcfWin, usize::MAX, *state, vcf_moves, Score::MIN, Score::MAX)
        .map(|mut sequence| {
            sequence.reverse();
            sequence
        })
}

fn vcf<ACC: EndgameAccumulator>(
    td: &mut ThreadData<impl ThreadType, impl Evaluator>,
    dest: impl VcfDestination,
    vcf_max_ply: usize,
    state: GameState,
    vcf_moves: VcfMovesUnchecked,
    alpha: Score, beta: Score,
) -> ACC {
    match state.board.player_color {
        Color::Black => try_vcf::<{ Color::Black }, _, ACC>(td, dest, vcf_max_ply, state, vcf_moves, alpha, beta),
        Color::White => try_vcf::<{ Color::White }, _, ACC>(td, dest, vcf_max_ply, state, vcf_moves, alpha, beta),
    }
}

fn try_vcf<const C: Color, TH: ThreadType, ACC: EndgameAccumulator>(
    td: &mut ThreadData<TH, impl Evaluator>,
    dest: impl VcfDestination,
    vcf_max_ply: usize,
    mut state: GameState,
    mut vcf_moves: VcfMovesUnchecked,
    mut alpha: Score, mut beta: Score,
) -> ACC {
    td.clear_vcf_stack_mut();

    let mut vcf_ply = 0;
    let mut score = 0;
    let mut move_counter: usize = 0;

    #[inline]
    fn backtrace_frames<ACC: EndgameAccumulator>(
        td: &mut ThreadData<impl ThreadType, impl Evaluator>,
        board: Board,
        vcf_ply: usize,
        four_pos: Pos
    ) -> ACC {
        let win_score = Score::win_in(td.ply + vcf_ply);
        let lose_score = Score::lose_in(td.ply + vcf_ply);

        let mut result = ACC::unit(four_pos, win_score);
        let mut hash_key = board.hash_key;

        let opponent_color = !board.player_color;

        while let Some(frame) = td.pop_vcf_frame_mut() {
            hash_key = hash_key.set(opponent_color, frame.defend_pos);
            tt_store_vcf_lose(&td.tt, hash_key, td.ply + vcf_ply, lose_score, true);

            hash_key = hash_key.set(board.player_color, frame.four_pos);
            tt_store_vcf_win(&td.tt, hash_key, frame.four_pos, td.ply + vcf_ply, win_score, true);

            result = result.append_pos(frame.defend_pos, frame.four_pos);
        }

        result
    }

    'vcf_search: loop {
        'position_search: for (seq, &four_pos) in vcf_moves.moves[.. vcf_moves.top as usize].iter()
            .enumerate()
            .skip(move_counter)
        {
            if TH::IS_MAIN
                && td.should_check_limit()
                && td.search_limit_exceeded()
            {
                td.set_aborted_mut();
                return ACC::ZERO;
            }

            if td.is_aborted() {
                return ACC::ZERO;
            }

            let idx = four_pos.idx_usize();

            let player_pattern = state.board.patterns.field.get_ref::<C>()[idx];

            if C == Color::Black && player_pattern.is_forbidden() {
                continue 'position_search;
            }

            if player_pattern.has_open_four() {
                let total_ply = td.ply + vcf_ply;
                let win_score = Score::win_in(total_ply);

                tt_store_vcf_win(&td.tt, state.board.hash_key, four_pos, total_ply, win_score, false);

                return backtrace_frames(td, state.board, vcf_ply, four_pos);
            }

            state.board.set_mut(four_pos);
            td.batch_counter.increment_single_mut();
            vcf_ply += 1;

            let defend_pos = state.board.patterns.unchecked_five_pos.get_ref::<C>().unwrap();
            let tt_key = state.board.hash_key.set(C.reversed(), defend_pos);
            td.tt.prefetch(tt_key);

            let defend_pattern = state.board.patterns.field.get_reversed::<C>()[defend_pos.idx_usize()];
            let defend_four_count = defend_pattern.count_fours();
            let defend_is_forbidden = C == Color::White && defend_pattern.is_forbidden();

            if match C {
                Color::Black => defend_four_count == PatternCount::Multiple
                    || defend_pattern.has_open_four(),
                Color::White => defend_pattern.has_open_four()
                    && !defend_is_forbidden
            } || dest.conditional_abort(defend_pattern) {
                state.board.unset_mut(four_pos);
                vcf_ply -= 1;
                continue 'position_search;
            }

            if (C == Color::White && defend_is_forbidden) || (
                defend_four_count == PatternCount::Cold
                    && (player_pattern.has_three() || dest.additional_reached(four_pos))
            ) {
                let total_ply = td.ply + vcf_ply;
                let win_score = Score::win_in(total_ply);

                tt_store_vcf_win(&td.tt, state.board.hash_key, four_pos, total_ply, win_score, false);

                return backtrace_frames(td, state.board, vcf_ply, four_pos);
            }

            if state.board.stones + 2 >= pos::U8_BOARD_SIZE {
                score = 0;
                state.board.unset_mut(four_pos);
                vcf_ply -= 1;
                continue 'position_search;
            }

            if vcf_ply + 2 >= vcf_max_ply {
                score = td.evaluator.eval_value(&state);
                state.board.unset_mut(four_pos);
                vcf_ply -= 1;
                continue 'position_search;
            }

            alpha = alpha.max(Score::lose_in(td.ply + vcf_ply));
            beta = beta.min(Score::win_in(td.ply + vcf_ply));
            if alpha >= beta { // mate distance pruning
                score = alpha;
                state.board.unset_mut(four_pos);
                vcf_ply -= 1;
                continue 'position_search;
            }

            if let Some(entry) = td.tt.probe(tt_key) {
                if entry.tt_flag.endgame_flag() == EndgameFlag::Cold { // endgame-cold pruning
                    state.board.unset_mut(four_pos);
                    vcf_ply -= 1;
                    continue 'position_search;
                }
            }

            state.board.set_mut(defend_pos);
            td.batch_counter.increment_single_mut();
            vcf_ply += 1;

            if state.board.patterns.counts.global.get_ref::<C>().total_fours() == 0 { // cold branch pruning
                state.board.unset_mut(defend_pos);
                state.board.unset_mut(four_pos);
                vcf_ply -= 2;
                continue 'position_search;
            }

            td.push_vcf_frame_mut(VcfFrame {
                vcf_moves,
                next_move_counter: seq + 1,
                four_pos,
                defend_pos,
            });

            if defend_four_count != PatternCount::Cold {
                let defend_move = state.board.patterns.unchecked_five_pos.get_reversed_ref::<C>().unwrap();

                if !state.board.patterns.field.get_ref::<C>()[defend_move.idx_usize()].has_any_four()
                    || (C == Color::Black && defend_pattern.is_forbidden())
                {
                    td.vcf_stack_top -= 1;
                    state.board.unset_mut(defend_pos);
                    state.board.unset_mut(four_pos);
                    vcf_ply -= 2;
                    continue 'position_search;
                }

                vcf_moves = VcfMovesUnchecked::unit(
                    state.board.patterns.unchecked_five_pos.get_reversed_ref::<C>().unwrap()
                );
            } else {
                vcf_moves = generate_vcf_moves(&state.board, ACC::DISTANCE_WINDOW, four_pos)
            }

            move_counter = 0;

            continue 'vcf_search;
        }

        let tt_entry = td.tt.probe(state.board.hash_key)
            .map(|mut tt_entry| {
                tt_entry.tt_flag.set_endgame_flag(EndgameFlag::Cold);
                tt_entry
            })
            .unwrap_or_else(|| TTEntry {
                best_move: MaybePos::NONE,
                depth: (td.ply + vcf_ply) as u8,
                age: td.tt.age,
                tt_flag: TTFlag::new(ScoreKind::Exact, EndgameFlag::Cold, false),
                score: 0,
                eval: 0,
            });

        td.tt.store_entry(state.board.hash_key, tt_entry);

        if let Some(frame) = td.pop_vcf_frame_mut() {
            state.board.unset_mut(frame.defend_pos);
            state.board.unset_mut(frame.four_pos);

            vcf_ply -= 2;

            vcf_moves = frame.vcf_moves;
            move_counter = frame.next_move_counter;
        } else {
            break 'vcf_search;
        }
    }

    ACC::ZERO
}

#[inline]
fn tt_store_vcf_win(
    tt: &TTView,
    hash_key: HashKey,
    four_pos: Pos,
    total_ply: usize,
    score: Score,
    is_pv: bool,
) {
    tt.store(
        hash_key,
        four_pos.into(),
        ScoreKind::LowerBound,
        EndgameFlag::Win,
        total_ply as u8,
        score,
        score,
        is_pv,
    )
}

#[inline]
fn tt_store_vcf_lose(
    tt: &TTView,
    hash_key: HashKey,
    total_ply: usize,
    score: Score,
    is_pv: bool,
) {
    tt.store(
        hash_key,
        MaybePos::NONE,
        ScoreKind::UpperBound,
        EndgameFlag::Lose,
        total_ply as u8,
        score,
        score,
        is_pv,
    )
}
