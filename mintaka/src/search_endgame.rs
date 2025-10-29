use crate::eval::evaluator::Evaluator;
use crate::game_state::GameState;
use crate::memo::transposition_table::TTView;
use crate::memo::tt_entry::{ScoreKind, TTFlag};
use crate::movegen::move_generator::generate_vcf_moves;
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use crate::value::Depth;
use rusty_renju::board::Board;
use rusty_renju::chebyshev_distance;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::score::{Score, Scores};
use rusty_renju::pattern::{Pattern, PatternCount};

pub trait EndgameAccumulator {

    const DISTANCE_WINDOW: isize;

    const ZERO: Self;

    fn unit(pos: Pos, score: Score) -> Self;

    fn append_pos(self, defend: Pos, threat: Pos) -> Self;

    fn score(&self) -> Score;

}

pub type SequenceEndgameAccumulator = Option<Vec<MaybePos>>;

impl EndgameAccumulator for SequenceEndgameAccumulator {

    const DISTANCE_WINDOW: isize = 5;

    const ZERO: Self = None;

    fn unit(pos: Pos, _score: Score) -> Self {
        Some(vec![pos.into()])
    }

    fn append_pos(self, defend: Pos, four: Pos) -> Self {
        self.map(|mut sequence| {
            sequence.push(defend.into());
            sequence.push(four.into());
            sequence
        })
    }

    fn score(&self) -> Score {
        0
    }

}

impl EndgameAccumulator for Score {

    const DISTANCE_WINDOW: isize = 5;

    const ZERO: Self = 0;

    fn unit(_pos: Pos, score: Score) -> Self {
        score
    }

    fn append_pos(self, _defend: Pos, _four: Pos) -> Self {
        self
    }

    fn score(&self) -> Score {
        *self
    }

}

pub const ENDGAME_MAX_MOVES: usize = 31;

#[derive(Debug, Copy, Clone)]
pub struct EndgameMovesUnchecked {
    pub moves: [MaybePos; ENDGAME_MAX_MOVES],
    pub top: u8,
}

impl EndgameMovesUnchecked {

    pub fn unit(pos: Pos) -> Self {
        Self {
            moves: {
                const EMPTY_MOVES: [MaybePos; ENDGAME_MAX_MOVES] = [MaybePos::NONE; ENDGAME_MAX_MOVES];

                let mut new_moves = EMPTY_MOVES;
                new_moves[0] = pos.into();
                new_moves
            },
            top: 1,
        }
    }

    pub fn init(&mut self) {
        self.top = 0;
    }

    pub fn next(&mut self) -> Option<Pos> {
        if self.top == 32 {
            return None;
        }

        let next_move = self.moves[self.top as usize].into();
        self.top += 1;
        next_move
    }

    pub fn sort_moves(&mut self, ref_pos: Pos) {
        let ref_row = ref_pos.row() as i16;
        let ref_col = ref_pos.col() as i16;

        self.moves[..self.top as usize].sort_by_key(|&pos| {
            chebyshev_distance!(ref_row, ref_col, pos.unwrap_unchecked().row() as i16, pos.unwrap_unchecked().col() as i16)
        });
    }

    pub fn is_empty(&self) -> bool {
        self.top == 0
    }

}

#[derive(Copy, Clone)]
pub struct EndgameFrame {
    pub moves: EndgameMovesUnchecked,
    pub alpha: Score,
    pub four_pos: Pos,
    pub defend_pos: Pos,
}

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

pub fn vcf_search<const R: RuleKind>(
    td: &mut ThreadData<impl ThreadType, impl Evaluator>,
    max_vcf_ply: Depth,
    state: &GameState,
    alpha: Score,
    beta: Score,
    static_eval: Score
) -> Score {
    if state.board.patterns.counts.global[state.board.player_color].total_fours() == 0 {
        return static_eval;
    }

    let mut vcf_moves = generate_vcf_moves(
        &state.board,
        Score::DISTANCE_WINDOW,
        state.history.recent_player_action().unwrap_or(pos::CENTER)
    );

    if vcf_moves.is_empty() {
        return static_eval;
    }

    vcf_moves.sort_moves(state.history.recent_player_action().unwrap_or(pos::CENTER));
    vcf_moves.init();

    vcf::<R, Score>(td, VcfWin, max_vcf_ply, *state, vcf_moves, alpha, beta, static_eval)
}

pub fn vcf_sequence<const R: RuleKind>(
    td: &mut ThreadData<impl ThreadType, impl Evaluator>,
    state: &GameState
) -> Option<Vec<MaybePos>> {
    let mut vcf_moves = generate_vcf_moves(&state.board, 8, pos::CENTER);

    if vcf_moves.is_empty() {
        return None;
    }

    vcf_moves.init();

    vcf::<R, SequenceEndgameAccumulator>(td, VcfWin, Depth::MAX, *state, vcf_moves, Score::MIN, Score::MAX, None)
        .map(|mut sequence| {
            sequence.reverse();
            sequence
        })
}

fn vcf<const R: RuleKind, ACC: EndgameAccumulator>(
    td: &mut ThreadData<impl ThreadType, impl Evaluator>,
    dest: impl VcfDestination,
    vcf_max_depth: Depth,
    state: GameState,
    vcf_moves: EndgameMovesUnchecked,
    alpha: Score, beta: Score,
    acc: ACC
) -> ACC {
    match state.board.player_color {
        Color::Black => try_vcf::<R, { Color::Black }, _, ACC>(td, dest, vcf_max_depth, state, vcf_moves, alpha, beta, acc),
        Color::White => try_vcf::<R, { Color::White }, _, ACC>(td, dest, vcf_max_depth, state, vcf_moves, alpha, beta, acc),
    }
}

fn try_vcf<const R: RuleKind, const C: Color, TH: ThreadType, ACC: EndgameAccumulator>(
    td: &mut ThreadData<TH, impl Evaluator>,
    dest: impl VcfDestination,
    mut vcf_depth_left: Depth,
    mut state: GameState,
    mut vcf_moves: EndgameMovesUnchecked,
    mut alpha: Score, beta: Score,
    acc: ACC
) -> ACC {
    td.clear_endgame_stack();

    let mut vcf_ply = 0;
    let mut move_counter: usize = 0;

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

        while let Some(frame) = td.pop_endgame_frame() {
            hash_key = hash_key.set(opponent_color, frame.defend_pos);
            tt_store_vcf_lose(&td.tt, hash_key, frame.defend_pos, lose_score);

            hash_key = hash_key.set(board.player_color, frame.four_pos);
            tt_store_vcf_win(&td.tt, hash_key, frame.four_pos, win_score);

            result = result.append_pos(frame.defend_pos, frame.four_pos);
        }

        result
    }

    'vcf_search: loop {
        'position_search: while let Some(four_pos) = vcf_moves.next() {
            if TH::IS_MAIN
                && td.should_check_limit()
                && td.search_limit_exceeded()
            {
                td.set_aborted();
                return ACC::ZERO;
            }

            if td.is_aborted() {
                return ACC::ZERO;
            }

            let idx = four_pos.idx_usize();

            let player_pattern = state.board.patterns.field.get_ref::<C>()[idx];

            if C == Color::Black && state.board.patterns.is_forbidden(four_pos) {
                continue 'position_search;
            }

            if player_pattern.has_open_four() {
                let total_ply = td.ply + vcf_ply;
                let win_score = Score::win_in(total_ply);

                tt_store_vcf_win(&td.tt, state.board.hash_key, four_pos, win_score);

                return backtrace_frames(td, state.board, vcf_ply, four_pos);
            }

            state.board.set_mut(four_pos);
            td.batch_counter.increment_single();
            vcf_ply += 1;
            vcf_depth_left -= 1;

            let defend_pos = state.board.patterns.unchecked_five_pos.get_ref::<C>().unwrap();
            let tt_key = state.board.hash_key.set(C.reversed(), defend_pos);
            td.tt.prefetch(tt_key);

            let defend_pattern = state.board.patterns.field.get_reversed::<C>()[defend_pos.idx_usize()];
            let defend_four_count = defend_pattern.count_fours();
            let defend_is_forbidden = R == RuleKind::Renju
                && C == Color::White
                && state.board.patterns.is_forbidden(defend_pos);

            if match (R, C) {
                (RuleKind::Renju, Color::Black) => defend_four_count == PatternCount::Multiple
                    || defend_pattern.has_open_four(),
                _ => defend_pattern.has_open_four() && !defend_is_forbidden
            } || dest.conditional_abort(defend_pattern) {
                state.board.unset_mut(four_pos);
                vcf_ply -= 1;
                vcf_depth_left += 1;
                continue 'position_search;
            }

            if (C == Color::White && defend_is_forbidden) || (
                defend_four_count == PatternCount::Cold
                    && (player_pattern.has_three() || dest.additional_reached(four_pos))
            ) {
                let total_ply = td.ply + vcf_ply;
                let win_score = Score::win_in(total_ply);

                tt_store_vcf_win(&td.tt, state.board.hash_key, four_pos, win_score);

                return backtrace_frames(td, state.board, vcf_ply, four_pos);
            }

            let mut alpha = alpha.max(Score::lose_in(td.ply + vcf_ply));
            let mut beta = beta.min(Score::win_in(td.ply + vcf_ply));

            if alpha >= beta // mate distance pruning
                || state.board.stones + 2 >= pos::U8_BOARD_SIZE
                || vcf_depth_left <= 0
            {
                state.board.unset_mut(four_pos);
                vcf_ply -= 1;
                vcf_depth_left += 1;
                continue 'position_search;
            }

            if let Some(entry) = td.tt.probe_entry(tt_key) {
                let mut abort = false;

                let tt_vcf_depth = entry.tt_flag.endgame_depth();

                if tt_vcf_depth != 0 { // tt pruning
                    if Score::is_winning(entry.score as Score) {
                        let total_ply = td.ply + vcf_ply;
                        let win_score = Score::win_in(total_ply);

                        tt_store_vcf_win(&td.tt, state.board.hash_key, four_pos, win_score);

                        return backtrace_frames(td, state.board, vcf_ply, four_pos);
                    } else if vcf_depth_left.min(TTFlag::MAX_TT_ENDGAME_DEPTH) <= tt_vcf_depth {
                        abort = true;
                    }
                }

                if abort {
                    state.board.unset_mut(four_pos);
                    vcf_ply -= 1;
                    vcf_depth_left += 1;
                    continue 'position_search;
                }
            }

            state.board.set_mut(defend_pos);
            td.batch_counter.increment_single();
            vcf_ply += 1;

            if state.board.patterns.counts.global.get_ref::<C>().total_fours() == 0 { // cold branch pruning
                state.board.unset_mut(defend_pos);
                state.board.unset_mut(four_pos);
                vcf_ply -= 2;
                vcf_depth_left += 1;
                continue 'position_search;
            }

            td.push_endgame_frame(EndgameFrame {
                moves: vcf_moves,
                alpha,
                four_pos,
                defend_pos,
            });

            if defend_four_count != PatternCount::Cold {
                let defend_move = state.board.patterns.unchecked_five_pos.get_reversed_ref::<C>().unwrap();

                if !state.board.patterns.field.get_ref::<C>()[defend_move.idx_usize()].has_any_four()
                    || (C == Color::Black && state.board.patterns.is_forbidden(defend_move))
                {
                    td.endgame_stack_top -= 1;
                    state.board.unset_mut(defend_pos);
                    state.board.unset_mut(four_pos);
                    vcf_ply -= 2;
                    vcf_depth_left += 1;
                    continue 'position_search;
                }

                vcf_moves = EndgameMovesUnchecked::unit(defend_move);
            } else {
                vcf_moves = generate_vcf_moves(&state.board, ACC::DISTANCE_WINDOW, four_pos)
            }

            vcf_moves.init();
            move_counter = 0;

            continue 'vcf_search;
        }

        if let Some(mut entry) = td.tt.probe_entry(state.board.hash_key)
            && entry.tt_flag.endgame_depth() <= vcf_depth_left
        {
            entry.tt_flag.set_endgame_depth(vcf_depth_left);

            td.tt.store_entry(state.board.hash_key, entry);
        } else {
            td.tt.store(
                state.board.hash_key,
                MaybePos::NONE,
                None,
                vcf_depth_left,
                0,
                Score::NAN,
                0,
                false,
            );
        }

        if let Some(frame) = td.pop_endgame_frame() {
            state.board.unset_mut(frame.defend_pos);
            state.board.unset_mut(frame.four_pos);

            vcf_ply -= 2;
            vcf_depth_left += 1;

            vcf_moves = frame.moves;
            alpha = frame.alpha;
        } else {
            break 'vcf_search;
        }
    }

    acc
}

fn tt_store_vcf_win(
    tt: &TTView,
    hash_key: HashKey,
    four_pos: Pos,
    score: Score,
) {
    tt.store(
        hash_key,
        four_pos.into(),
        Some(ScoreKind::Exact),
        TTFlag::MAX_TT_ENDGAME_DEPTH,
        0,
        score,
        score,
        false,
    )
}

#[inline]
fn tt_store_vcf_lose(
    tt: &TTView,
    hash_key: HashKey,
    defend_pos: Pos,
    score: Score,
) {
    tt.store(
        hash_key,
        defend_pos.into(),
        Some(ScoreKind::Exact),
        TTFlag::MAX_TT_ENDGAME_DEPTH,
        0,
        score,
        score,
        false,
    )
}
