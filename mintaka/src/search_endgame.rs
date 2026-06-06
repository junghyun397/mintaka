use std::cmp::Reverse;
use rusty_renju::board::Board;
use crate::eval::evaluator::Evaluator;
use crate::game_state::GameState;
use crate::memo::transposition_table;
use crate::memo::transposition_table::TTView;
use crate::memo::tt_entry::{ScoreKind, TTFlag};
use crate::movegen::move_generator::generate_endgame_moves;
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use crate::value::Depth;
use rusty_renju::hash_key::HashKey;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::score::{Score, Scores};
use rusty_renju::pattern::Pattern;

pub trait SequenceTracker {
    type Output;

    fn unit(four: Pos) -> Self;

    fn push(&mut self, response: Pos, attack: Pos);

    fn resolve(self, score: Score) -> Self::Output;

    fn fallback(score: Score) -> Self::Output;
}

pub struct NullSequenceTracker; impl SequenceTracker for NullSequenceTracker {
    type Output = Score;

    fn unit(_four: Pos) -> Self { Self }

    fn push(&mut self, _response: Pos, _attack: Pos) { }

    fn resolve(self, score: Score) -> Self::Output {
        score
    }

    fn fallback(score: Score) -> Self::Output {
        score
    }
}

type VecSequenceTracker = Vec<Pos>; impl SequenceTracker for VecSequenceTracker {
    type Output = Option<Vec<Pos>>;

    fn unit(four: Pos) -> Self {
        vec![four]
    }

    fn push(&mut self, response: Pos, attack: Pos) {
        self.push(response);
        self.push(attack);
    }

    fn resolve(self, _score: Score) -> Self::Output {
        Some(self)
    }

    fn fallback(_score: Score) -> Self::Output {
        None
    }
}

pub const ENDGAME_MAX_MOVES: usize = 30;

#[derive(Debug, Copy, Clone)]
pub struct EndgameMovesUnchecked {
    pub moves: [MaybePos; ENDGAME_MAX_MOVES],
    pub top: u8,
}

impl EndgameMovesUnchecked {
    pub const EMPTY: Self = Self {
        moves: [MaybePos::NONE; ENDGAME_MAX_MOVES],
        top: 0,
    };

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
        if self.top == ENDGAME_MAX_MOVES as u8 {
            return None;
        }

        let next_move = self.moves[self.top as usize].into();
        self.top += 1;
        next_move
    }

    pub fn sort_moves<const R: RuleKind>(&mut self, board: &Board<R>, ref_pos: Pos) {
        self.moves[..self.top as usize].sort_by_key(|pos| {
            let pos = pos.unwrap();

            let potential_score = board.patterns.field[board.player_color][pos.idx_usize()]
                .count_potential_four()
                .min(2);

            Reverse(potential_score * 10 + (pos::BOARD_WIDTH as u32 - pos.distance(ref_pos) as u32) * 3)
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
    pub response_pos: Pos,
}

impl EndgameFrame {
    pub const EMPTY: Self = Self {
        moves: EndgameMovesUnchecked::EMPTY,
        alpha: Score::NAN,
        four_pos: MaybePos::INVALID_POS,
        response_pos: MaybePos::INVALID_POS,
    };
}

pub trait VcfDestination {
    fn conditional_abort(&self, response_pattern: Pattern) -> bool;

    fn additional_reached(&self, four_pos: Pos) -> bool;
}

pub struct VcfWin; impl VcfDestination for VcfWin {
    fn conditional_abort(&self, _: Pattern) -> bool {
        false
    }

    fn additional_reached(&self, _: Pos) -> bool {
        false
    }
}

pub struct VcfCounter {
    target_pos: Pos
}

impl VcfDestination for VcfCounter {
    fn conditional_abort(&self, response_pattern: Pattern) -> bool {
        response_pattern.has_open_three()
    }

    fn additional_reached(&self, four_pos: Pos) -> bool {
        self.target_pos == four_pos
    }
}

pub fn endgame_search<const R: RuleKind, const VCT: bool>(
    td: &mut ThreadData<R, impl ThreadType, impl Evaluator<R>>,
    max_ply: Depth,
    state: &GameState<R>,
    alpha: Score,
    beta: Score,
    static_eval: Score,
) -> Score {
    let indexes = state.board.patterns.indexes[state.board.player_color];

    let empty_closed_fours = indexes.closed_fours.is_empty();

    if (!VCT && empty_closed_fours)
        || (VCT && empty_closed_fours && indexes.open_threes.is_empty())
    {
        return static_eval;
    }

    let recent_player_action = state.history.previous_action().unwrap_or(pos::CENTER);

    let mut endgame_moves = generate_endgame_moves::<R, VCT>(&state.board, 8, recent_player_action);

    if endgame_moves.is_empty() {
        return static_eval;
    }

    endgame_moves.sort_moves(&state.board, recent_player_action);
    endgame_moves.init();

    if VCT {
        todo!()
    } else {
        vcf::<R, 5, NullSequenceTracker>(
            td, VcfWin, max_ply,
            *state, endgame_moves,
            static_eval, alpha, beta
        )
    }
}

pub fn endgame_sequence<const R: RuleKind, const VCT: bool>(
    td: &mut ThreadData<R, impl ThreadType, impl Evaluator<R>>,
    state: &GameState<R>
) -> Option<Vec<Pos>> {
    let mut endgame_moves = generate_endgame_moves::<R, VCT>(&state.board, 5, pos::CENTER);

    if endgame_moves.is_empty() {
        return None;
    }

    endgame_moves.init();

    let maybe_sequence = if VCT {
        todo!()
    } else {
        vcf::<R, 5, VecSequenceTracker>(
            td, VcfWin, Depth::MAX,
            *state, endgame_moves,
            0, Score::MIN, Score::MAX
        )
    };

    maybe_sequence.map(|mut sq| {
        sq.reverse();
        sq
    })
}

fn vcf<const R: RuleKind, const DW: u8, Sq: SequenceTracker>(
    td: &mut ThreadData<R, impl ThreadType, impl Evaluator<R>>,
    dest: impl VcfDestination,
    max_depth: Depth,
    state: GameState<R>,
    vcf_moves: EndgameMovesUnchecked,
    static_eval: Score,
    alpha: Score, beta: Score,
) -> Sq::Output {
    match state.board.player_color {
        Color::Black => try_vcf::<R, { Color::Black }, DW, _, Sq>(td, dest, max_depth, state, vcf_moves, static_eval, alpha, beta),
        Color::White => try_vcf::<R, { Color::White }, DW, _, Sq>(td, dest, max_depth, state, vcf_moves, static_eval, alpha, beta),
    }
}

fn try_vcf<const R: RuleKind, const C: Color, const DW: u8, TH: ThreadType, Sq: SequenceTracker>(
    td: &mut ThreadData<R, TH, impl Evaluator<R>>,
    dest: impl VcfDestination,
    mut vcf_depth_left: Depth,
    mut state: GameState<R>,
    mut vcf_moves: EndgameMovesUnchecked,
    static_eval: Score,
    mut alpha: Score, beta: Score,
) -> Sq::Output {
    td.clear_endgame_stack();

    let mut vcf_ply = 0;

    fn backtrace_frames<const R: RuleKind, Sq: SequenceTracker>(
        td: &mut ThreadData<R, impl ThreadType, impl Evaluator<R>>,
        mut hash_key: HashKey,
        player_color: Color,
        vcf_ply: usize,
        four_pos: Pos
    ) -> Sq {
        let total_ply = td.ply + vcf_ply;

        let win_score = Score::win_in(total_ply);
        let lose_score = Score::lose_in(total_ply);

        let mut result = Sq::unit(four_pos);

        while let Some(frame) = td.pop_endgame_frame() {
            hash_key = hash_key.set(player_color, frame.response_pos);
            tt_store_vcf_lose(&td.tt, hash_key, frame.response_pos, lose_score, total_ply);

            hash_key = hash_key.set(!player_color, frame.four_pos);
            tt_store_vcf_win(&td.tt, hash_key, frame.four_pos, win_score, total_ply);

            result.push(frame.response_pos, frame.four_pos);
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
                return Sq::fallback(static_eval);
            }

            if td.is_aborted() {
                return Sq::fallback(static_eval);
            }

            let idx = four_pos.idx_usize();

            let player_pattern = state.board.patterns.field[C][idx];

            if C == Color::Black && state.board.patterns.is_forbidden(four_pos) {
                continue 'position_search;
            }

            if player_pattern.has_open_four() {
                let total_ply = td.ply + vcf_ply;
                let win_score = Score::win_in(total_ply);

                tt_store_vcf_win(&td.tt, state.board.hash_key, four_pos, win_score, total_ply);

                let trace_result = backtrace_frames(td, state.board.hash_key, !state.board.player_color, vcf_ply, four_pos);

                return Sq::resolve(trace_result, win_score);
            }

            let parent_hash_key = state.board.hash_key;
            let parent_player_color = state.board.player_color;

            td.batch_counter.increment();
            let artifact = state.board.set_mut(four_pos);
            td.evaluator.play(&state.board, artifact, four_pos.into());
            vcf_ply += 1;
            vcf_depth_left -= 1;

            let response_pos = state.board.patterns.five_pos[C].unwrap();
            let tt_key = state.board.hash_key.set(C.reversed(), response_pos);
            td.tt.prefetch(tt_key);

            let response_pattern = state.board.patterns.field[!C][response_pos.idx_usize()];
            let response_four_count = response_pattern.count_any_fours();
            let response_is_forbidden = R == RuleKind::Renju && C == Color::White
                && state.board.patterns.is_forbidden(response_pos);

            if match (R, C) {
                (RuleKind::Renju, Color::Black) => response_four_count > 1
                    || response_pattern.has_open_four(),
                _ => response_pattern.has_open_four() && !response_is_forbidden
            } || dest.conditional_abort(response_pattern) {
                let artifact = state.board.unset_mut(four_pos);
                td.evaluator.undo(&state.board, artifact, four_pos.into());
                vcf_ply -= 1;
                vcf_depth_left += 1;
                continue 'position_search;
            }

            if (C == Color::White && response_is_forbidden) || (
                response_four_count == 0
                    && (player_pattern.has_open_three() || dest.additional_reached(four_pos))
            ) {
                let total_ply = td.ply + vcf_ply;
                let win_score = Score::win_in(total_ply);

                tt_store_vcf_win(&td.tt, parent_hash_key, four_pos, win_score, total_ply);

                let trace_result = backtrace_frames(td, parent_hash_key, parent_player_color, vcf_ply, four_pos);

                return Sq::resolve(trace_result, win_score);
            }

            let alpha = alpha.max(Score::lose_in(td.ply + vcf_ply));
            let beta = beta.min(Score::win_in(td.ply + vcf_ply));

            if alpha >= beta // mate distance pruning
                || state.board.stones + 2 >= pos::U8_BOARD_SIZE
                || vcf_depth_left <= 0
            {
                let artifact = state.board.unset_mut(four_pos);
                td.evaluator.undo(&state.board, artifact, four_pos.into());
                vcf_ply -= 1;
                vcf_depth_left += 1;
                continue 'position_search;
            }

            if let Some(entry) = td.tt.probe(tt_key) {
                // tt cutoff
                if Score::is_winning(entry.score as Score) {
                    let total_ply = td.ply + vcf_ply;
                    let win_score = Score::win_in(total_ply);

                    tt_store_vcf_win(&td.tt, parent_hash_key, four_pos, win_score, total_ply);

                    let trace_result = backtrace_frames(td, parent_hash_key, parent_player_color, vcf_ply, four_pos);

                    return Sq::resolve(trace_result, win_score);
                }

                let tt_endgame_depth = entry.tt_flag.endgame_depth();

                // tt vcf cache
                if vcf_depth_left.min(TTFlag::MAX_TT_ENDGAME_DEPTH as Depth) as u8 <= tt_endgame_depth {
                    let artifact = state.board.unset_mut(four_pos);
                    td.evaluator.undo(&state.board, artifact, four_pos.into());
                    vcf_ply -= 1;
                    vcf_depth_left += 1;
                    continue 'position_search;
                }
            }

            td.batch_counter.increment();
            let artifact = state.board.set_mut(response_pos);
            td.evaluator.play(&state.board, artifact, response_pos.into());
            vcf_ply += 1;

            if !state.board.patterns.indexes[C].has_any_four() { // cold branch pruning
                let artifact = state.board.unset_mut(response_pos);
                td.evaluator.undo(&state.board, artifact, response_pos.into());
                let artifact = state.board.unset_mut(four_pos);
                td.evaluator.undo(&state.board, artifact, four_pos.into());
                vcf_ply -= 2;
                vcf_depth_left += 1;
                continue 'position_search;
            }

            td.push_endgame_frame(EndgameFrame {
                moves: vcf_moves,
                alpha,
                four_pos,
                response_pos,
            });

            if response_four_count != 0 {
                let response_move = state.board.patterns.five_pos[!C].unwrap();

                if !state.board.patterns.field[C][response_move.idx_usize()].has_any_four()
                    || (C == Color::Black && state.board.patterns.is_forbidden(response_move))
                {
                    td.endgame_stack_top -= 1;
                    let artifact = state.board.unset_mut(response_pos);
                    td.evaluator.undo(&state.board, artifact, response_pos.into());
                    let artifact = state.board.unset_mut(four_pos);
                    td.evaluator.undo(&state.board, artifact, four_pos.into());
                    vcf_ply -= 2;
                    vcf_depth_left += 1;
                    continue 'position_search;
                }

                vcf_moves = EndgameMovesUnchecked::unit(response_move);
            } else {
                vcf_moves = generate_endgame_moves::<R, false>(&state.board, DW, four_pos);
            }

            vcf_moves.init();

            continue 'vcf_search;
        }

        let clamped_vcf_depth_left = vcf_depth_left.min(TTFlag::MAX_TT_ENDGAME_DEPTH as Depth) as u8;

        if let Some(mut entry) = td.tt.probe(state.board.hash_key) {
            if !entry.tt_flag.is_endgame_proven()
                && entry.tt_flag.endgame_depth() <= clamped_vcf_depth_left
            {
                entry.tt_flag.set_endgame_depth(clamped_vcf_depth_left);

                td.tt.store_entry(state.board.hash_key, entry);
            }
        } else {
            td.tt.store(
                state.board.hash_key,
                MaybePos::NONE,
                None,
                clamped_vcf_depth_left,
                0,
                Score::NAN,
                0,
                false,
            );
        }

        if let Some(frame) = td.pop_endgame_frame() {
            let artifact = state.board.unset_mut(frame.response_pos);
            td.evaluator.undo(&state.board, artifact, frame.response_pos.into());
            let artifact = state.board.unset_mut(frame.four_pos);
            td.evaluator.undo(&state.board, artifact, frame.four_pos.into());
            vcf_ply -= 2;
            vcf_depth_left += 1;

            vcf_moves = frame.moves;
            alpha = frame.alpha;
        } else {
            break 'vcf_search;
        }
    }

    Sq::fallback(static_eval)
}

fn tt_store_vcf_win(
    tt: &TTView,
    hash_key: HashKey,
    four_pos: Pos,
    score: Score,
    ply: usize,
) {
    tt.store_endgame_proven(
        hash_key,
        four_pos,
        ScoreKind::LowerBound,
        transposition_table::encode_mate_distance(score, ply),
        false,
    );
}

#[inline]
fn tt_store_vcf_lose(
    tt: &TTView,
    hash_key: HashKey,
    response_pos: Pos,
    score: Score,
    ply: usize,
) {
    tt.store_endgame_proven(
        hash_key,
        response_pos,
        ScoreKind::UpperBound,
        transposition_table::encode_mate_distance(score, ply),
        false,
    )
}
