use crate::eval::evaluator::Evaluator;
use crate::game_state::GameState;
use crate::memo::transposition_table;
use crate::memo::transposition_table::TTView;
use crate::memo::tt_entry::{ScoreKind, TTEntry, TTEntryBucketProbe};
use crate::movegen::move_generator::generate_endgame_moves;
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use crate::utils::depth::Depth;
use rusty_renju::board::Board;
use rusty_renju::hash_key::HashKey;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::score::{MaybeScore, Score};
use rusty_renju::notation::pos;
use rusty_renju::pattern::Pattern;
use std::cmp::Reverse;

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
    max_ply: u8,
    state: &GameState<R>,
    alpha: Score, beta: Score,
    static_eval: Score,
    is_pv: bool,
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
        vcf::<R, 5, ScoreProof>(
            td, VcfWin, max_ply,
            *state, endgame_moves,
            alpha, beta,
            is_pv,
        ).map_or(static_eval, |proof| proof.score())
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

    if VCT {
        todo!()
    } else {
        vcf::<R, 5, SequenceProof>(
            td, VcfWin, pos::U8_BOARD_SIZE,
            *state, endgame_moves,
            Score::NEG_INF, Score::INF,
            true,
        ).map(SequenceProof::into_sequence)
    }
}

struct EndgameContext<D> {
    dest: D,
    beta: Score,
    is_pv: bool,
}

trait EndgameProof {
    fn new(four_pos: Pos, ply: usize) -> Self;

    fn push_pair(&mut self, response: Pos, attack: Pos);

    fn ply(&self) -> usize;

    fn score(&self) -> Score {
        Score::win_in(self.ply())
    }
}

#[derive(Copy, Clone)]
struct ScoreProof {
    ply: usize,
}

impl EndgameProof for ScoreProof {
    fn new(_four_pos: Pos, ply: usize) -> Self {
        Self { ply }
    }

    fn push_pair(&mut self, _response: Pos, _attack: Pos) { }

    fn ply(&self) -> usize {
        self.ply
    }
}

struct SequenceProof {
    sequence: Vec<Pos>,
    ply: usize,
}

impl EndgameProof for SequenceProof {
    fn new(four_pos: Pos, ply: usize) -> Self {
        Self { sequence: vec![four_pos], ply }
    }

    fn push_pair(&mut self, response: Pos, attack: Pos) {
        self.sequence.push(response);
        self.sequence.push(attack);
    }

    fn ply(&self) -> usize {
        self.ply
    }
}

impl SequenceProof {
    fn into_sequence(mut self) -> Vec<Pos> {
        self.sequence.reverse();
        self.sequence
    }
}

fn vcf<const R: RuleKind, const DW: u8, Pr: EndgameProof>(
    td: &mut ThreadData<R, impl ThreadType, impl Evaluator<R>>,
    dest: impl VcfDestination,
    max_depth: u8,
    mut state: GameState<R>,
    vcf_moves: EndgameMovesUnchecked,
    alpha: Score, beta: Score,
    is_pv: bool,
) -> Option<Pr> {
    let context = EndgameContext { dest, beta, is_pv };

    match state.board.player_color {
        Color::Black => try_vcf::<R, { Color::Black }, DW, _, Pr>(td, &context, max_depth, &mut state, vcf_moves, alpha, 0),
        Color::White => try_vcf::<R, { Color::White }, DW, _, Pr>(td, &context, max_depth, &mut state, vcf_moves, alpha, 0),
    }
}

fn try_vcf<const R: RuleKind, const C: Color, const DW: u8, TH: ThreadType, Pf: EndgameProof>(
    td: &mut ThreadData<R, TH, impl Evaluator<R>>,
    context: &EndgameContext<impl VcfDestination>,
    vcf_depth_left: u8,
    state: &mut GameState<R>,
    mut vcf_moves: EndgameMovesUnchecked,
    alpha: Score,
    vcf_ply: usize,
) -> Option<Pf> {
    while let Some(four_pos) = vcf_moves.next() {
        if TH::IS_MAIN
            && td.should_check_limit()
            && td.search_limit_exceeded()
        {
            td.set_aborted();
        }

        if td.is_aborted() {
            return None;
        }

        let player_pattern = state.board.patterns.field[C][four_pos.idx_usize()];

        if C == Color::Black && state.board.patterns.is_forbidden(four_pos) {
            continue;
        }

        if player_pattern.has_open_four() {
            let proof = Pf::new(four_pos, vcf_ply + 3);
            tt_store_vcf_win(&td.tt, state.board.hash_key, four_pos, proof.score(), proof.ply(), context.is_pv);
            return Some(proof);
        }

        let Some(child_depth) = vcf_depth_left.checked_sub(1) else {
            continue;
        };

        td.batch_counter.increment();
        let artifact = state.board.set_mut(four_pos);
        td.evaluator.play(&state.board, artifact, four_pos.into());

        let proof = 'candidate: {
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
            } || context.dest.conditional_abort(response_pattern) {
                break 'candidate None;
            }

            if (C == Color::White && response_is_forbidden) || (
                response_four_count == 0
                    && (player_pattern.has_open_three() || context.dest.additional_reached(four_pos))
            ) {
                break 'candidate Some(Pf::new(four_pos, vcf_ply + 1));
            }

            let ply_after_attack = td.ply + vcf_ply + 1;
            let child_alpha = alpha.max(Score::lose_in(ply_after_attack));

            if child_alpha >= context.beta
                || child_alpha >= Score::win_in(ply_after_attack)
                || state.board.stones + 2 >= pos::U8_BOARD_SIZE
                || child_depth == 0
            {
                break 'candidate None;
            }

            if let Some(TTEntryBucketProbe { entry, .. }) = td.tt.probe(tt_key) {
                let tt_score = MaybeScore::from(entry.score as i32);

                // tt cutoff
                if tt_score.is_some() && tt_score.unwrap().is_win() {
                    break 'candidate Some(Pf::new(four_pos, vcf_ply + 1));
                }

                // tt vcf cache
                if child_depth <= entry.endgame_depth {
                    break 'candidate None;
                }
            }

            td.batch_counter.increment();
            let artifact = state.board.set_mut(response_pos);
            td.evaluator.play(&state.board, artifact, response_pos.into());

            let child_moves = if !state.board.patterns.indexes[C].has_any_four() {
                None // cold branch pruning
            } else if response_four_count != 0 {
                let response_move = state.board.patterns.five_pos[!C].unwrap();

                if !state.board.patterns.field[C][response_move.idx_usize()].has_any_four()
                    || (C == Color::Black && state.board.patterns.is_forbidden(response_move))
                {
                    None
                } else {
                    Some(EndgameMovesUnchecked::unit(response_move))
                }
            } else {
                Some(generate_endgame_moves::<R, false>(&state.board, DW, four_pos))
            };

            let mut proof = if let Some(mut moves) = child_moves {
                moves.init();
                try_vcf::<R, C, DW, TH, Pf>(td, context, child_depth, state, moves, child_alpha, vcf_ply + 2)
            } else {
                None
            };

            let artifact = state.board.unset_mut(response_pos);
            td.evaluator.undo(&state.board, artifact, response_pos.into());

            if td.is_aborted() {
                break 'candidate None;
            }

            if let Some(proof) = &mut proof {
                tt_store_vcf_lose(&td.tt, state.board.hash_key, response_pos, -proof.score(), proof.ply(), context.is_pv);
                proof.push_pair(response_pos, four_pos);
            }

            proof
        };

        let artifact = state.board.unset_mut(four_pos);
        td.evaluator.undo(&state.board, artifact, four_pos.into());

        if td.is_aborted() {
            return None;
        }

        if let Some(proof) = proof {
            tt_store_vcf_win(&td.tt, state.board.hash_key, four_pos, proof.score(), proof.ply(), context.is_pv);
            return Some(proof);
        }
    }

    if td.is_aborted() {
        return None;
    }

    if let Some(TTEntryBucketProbe { slot, mut entry }) = td.tt.probe(state.board.hash_key) {
        if entry.endgame_depth != u8::MAX
            && entry.endgame_depth <= vcf_depth_left
        {
            entry.endgame_depth = vcf_depth_left;

            td.tt.update_entry(state.board.hash_key, slot, entry);
        }
    } else {
        td.tt.store(
            state.board.hash_key,
            MaybePos::NONE,
            Depth::ZERO,
            vcf_depth_left,
            None,
            MaybeScore::NONE,
            MaybeScore::NONE,
            false,
        );
    }

    None
}

fn tt_store_vcf_win(
    tt: &TTView,
    hash_key: HashKey,
    four_pos: Pos,
    score: Score,
    ply: usize,
    is_pv: bool,
) {
    tt.store(
        hash_key,
        four_pos.into(),
        Depth::ZERO,
        TTEntry::ENDGAME_PROVEN_DEPTH,
        Some(ScoreKind::LowerBound),
        score.into(),
        transposition_table::encode_mate_distance(score, ply).into(),
        is_pv,
    );
}

#[inline]
fn tt_store_vcf_lose(
    tt: &TTView,
    hash_key: HashKey,
    response_pos: Pos,
    score: Score,
    ply: usize,
    is_pv: bool,
) {
    tt.store(
        hash_key,
        response_pos.into(),
        Depth::ZERO,
        TTEntry::ENDGAME_PROVEN_DEPTH,
        Some(ScoreKind::UpperBound),
        score.into(),
        transposition_table::encode_mate_distance(score, ply).into(),
        is_pv,
    );
}
