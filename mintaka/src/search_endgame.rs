use crate::eval::evaluator::Evaluator;
use crate::state::GameState;
use crate::memo::transposition_table::TTView;
use crate::memo::tt_entry::{ScoreKind, TTFlag};
use crate::movegen::move_generator::generate_endgame_moves;
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use crate::value::Depth;
use rusty_renju::board::Board;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::score::{Score, Scores};
use rusty_renju::pattern::{Pattern, PatternCount};

pub trait SequenceTracker {
    type Output;

    fn unit(four: Pos) -> Self;

    fn push(&mut self, defend: Pos, attack: Pos);

    fn resolve(self, score: Score) -> Self::Output;

    fn fallback(score: Score) -> Self::Output;
}

pub struct NullSequenceTracker; impl SequenceTracker for NullSequenceTracker {
    type Output = Score;

    fn unit(_four: Pos) -> Self { Self }

    fn push(&mut self, _defend: Pos, _attack: Pos) { }

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

    fn push(&mut self, defend: Pos, attack: Pos) {
        self.push(defend);
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
        if self.top == 32 {
            return None;
        }

        let next_move = self.moves[self.top as usize].into();
        self.top += 1;
        next_move
    }

    pub fn sort_moves(&mut self, ref_pos: Pos) {
        self.moves[..self.top as usize].sort_by_key(|maybe_pos|
            maybe_pos.unwrap().distance(ref_pos)
        );
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

impl EndgameFrame {
    pub const EMPTY: Self = Self {
        moves: EndgameMovesUnchecked::EMPTY,
        alpha: Score::NAN,
        four_pos: MaybePos::INVALID_POS,
        defend_pos: MaybePos::INVALID_POS,
    };
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

pub fn endgame_search<const R: RuleKind, const VCT: bool>(
    td: &mut ThreadData<impl ThreadType, impl Evaluator>,
    max_ply: Depth,
    state: &GameState,
    static_eval: Score,
    alpha: Score,
    beta: Score,
) -> Score {
    let counts = state.board.patterns.counts.global[state.board.player_color];

    let mut total_threats = counts.total_fours();

    if VCT {
        total_threats += counts.threes;
    }

    if total_threats == 0 {
        return static_eval;
    }

    let recent_player_action = state.history.recent_player_action().unwrap_or(pos::CENTER);

    let mut endgame_moves = generate_endgame_moves::<VCT>(&state.board, 8, recent_player_action);

    if endgame_moves.is_empty() {
        return static_eval;
    }

    endgame_moves.sort_moves(state.history.recent_player_action().unwrap_or(pos::CENTER));
    endgame_moves.init();

    if VCT {
        pns::<R, 5, NullSequenceTracker>(
            td, max_ply,
            *state, endgame_moves,
            static_eval, alpha, beta
        )
    } else {
        vcf::<R, 5, NullSequenceTracker>(
            td, VcfWin, max_ply,
            *state, endgame_moves,
            static_eval, alpha, beta
        )
    }
}

pub fn endgame_sequence<const R: RuleKind, const VCT: bool>(
    td: &mut ThreadData<impl ThreadType, impl Evaluator>,
    state: &GameState
) -> Option<Vec<Pos>> {
    let mut endgame_moves = generate_endgame_moves::<VCT>(&state.board, 5, pos::CENTER);

    if endgame_moves.is_empty() {
        return None;
    }

    endgame_moves.init();

    let maybe_sequence = if VCT {
        pns::<R, 5, VecSequenceTracker>(
            td, Depth::MAX,
            *state, endgame_moves,
            0, Score::MIN, Score::MAX
        )
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

fn pns<const R: RuleKind, const DW: isize, Sq: SequenceTracker>(
    td: &mut ThreadData<impl ThreadType, impl Evaluator>,
    max_depth: Depth,
    state: GameState,
    endgame_moves: EndgameMovesUnchecked,
    static_eval: Score,
    alpha: Score, beta: Score,
) -> Sq::Output {
    match state.board.player_color {
        Color::Black => try_pns::<R, { Color::Black }, DW, _, Sq>(td, max_depth, max_depth, state, endgame_moves, static_eval, alpha, beta),
        Color::White => try_pns::<R, { Color::White }, DW, _, Sq>(td, max_depth, max_depth, state, endgame_moves, static_eval, alpha, beta),
    }
}

fn try_pns<const R: RuleKind, const C: Color, const DW: isize, TH: ThreadType, Sq: SequenceTracker>(
    td: &mut ThreadData<TH, impl Evaluator>,
    mut vcf_switch_depth: Depth,
    mut depth_left: Depth,
    mut state: GameState,
    mut endgame_moves: EndgameMovesUnchecked,
    static_eval: Score,
    mut alpha: Score, beta: Score,
) -> Sq::Output {
    todo!()
}

fn vcf<const R: RuleKind, const DW: u8, Sq: SequenceTracker>(
    td: &mut ThreadData<impl ThreadType, impl Evaluator>,
    dest: impl VcfDestination,
    max_depth: Depth,
    state: GameState,
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
    td: &mut ThreadData<TH, impl Evaluator>,
    dest: impl VcfDestination,
    mut vcf_depth_left: Depth,
    mut state: GameState,
    mut vcf_moves: EndgameMovesUnchecked,
    static_eval: Score,
    mut alpha: Score, beta: Score,
) -> Sq::Output {
    td.clear_endgame_stack();

    let mut vcf_ply = 0;

    fn backtrace_frames<Sq: SequenceTracker>(
        td: &mut ThreadData<impl ThreadType, impl Evaluator>,
        board: Board,
        vcf_ply: usize,
        four_pos: Pos
    ) -> Sq {
        let win_score = Score::win_in(td.ply + vcf_ply);
        let lose_score = Score::lose_in(td.ply + vcf_ply);

        let mut result = Sq::unit(four_pos);
        let mut hash_key = board.hash_key;

        let opponent_color = !board.player_color;

        while let Some(frame) = td.pop_endgame_frame() {
            hash_key = hash_key.set(opponent_color, frame.defend_pos);
            tt_store_vcf_lose(&td.tt, hash_key, frame.defend_pos, lose_score);

            hash_key = hash_key.set(board.player_color, frame.four_pos);
            tt_store_vcf_win(&td.tt, hash_key, frame.four_pos, win_score);

            result.push(frame.defend_pos, frame.four_pos);
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

                tt_store_vcf_win(&td.tt, state.board.hash_key, four_pos, win_score);

                let trace_result = backtrace_frames(td, state.board, vcf_ply, four_pos);

                return Sq::resolve(trace_result, win_score);
            }

            state.board.set_mut(four_pos);
            td.batch_counter.increment_single();
            vcf_ply += 1;
            vcf_depth_left -= 1;

            let defend_pos = state.board.patterns.unchecked_five_pos[C].unwrap();
            let tt_key = state.board.hash_key.set(C.reversed(), defend_pos);
            td.tt.prefetch(tt_key);

            let defend_pattern = state.board.patterns.field[!C][defend_pos.idx_usize()];
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

                let trace_result = backtrace_frames(td, state.board, vcf_ply, four_pos);

                return Sq::resolve(trace_result, win_score);
            }

            let mut alpha = alpha.max(Score::lose_in(td.ply + vcf_ply));
            let beta = beta.min(Score::win_in(td.ply + vcf_ply));

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

                        let trace_result = backtrace_frames(td, state.board, vcf_ply, four_pos);

                        return Sq::resolve(trace_result, win_score);
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

            if state.board.patterns.counts.global[C].total_fours() == 0 { // cold branch pruning
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
                let defend_move = state.board.patterns.unchecked_five_pos[!C].unwrap();

                if !state.board.patterns.field[C][defend_move.idx_usize()].has_any_four()
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
                vcf_moves = generate_endgame_moves::<false>(&state.board, DW, four_pos);
            }

            vcf_moves.init();

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

    Sq::fallback(static_eval)
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
