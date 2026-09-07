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

struct EndgameContext {
    beta: Score,
    is_pv: bool,
}

trait EndgameProof {
    const SEQUENCE: bool;

    fn entry(four_pos: Pos, ply: usize) -> Self;

    fn stand_pat(eval: Score) -> Self;

    fn abort() -> Self;

    fn score(&self) -> Score;

    fn push_pair(&mut self, response: Pos, attack: Pos);
}

impl EndgameProof for Score {
    const SEQUENCE: bool = false;

    fn entry(_four_pos: Pos, ply: usize) -> Self {
        Score::win_in(ply)
    }

    fn stand_pat(eval: Score) -> Self {
        eval
    }

    fn abort() -> Self {
        Score::DRAW
    }

    fn score(&self) -> Score {
        *self
    }

    fn push_pair(&mut self, _response: Pos, _attack: Pos) { }
}

struct SequenceProof {
    score: Score,
    sequence: Vec<Pos>,
}

impl EndgameProof for SequenceProof {
    const SEQUENCE: bool = true;

    fn entry(four_pos: Pos, ply: usize) -> Self {
        Self { score: Score::win_in(ply), sequence: vec![four_pos] }
    }

    fn stand_pat(eval: Score) -> Self {
        Self { score: eval, sequence: vec![] }
    }

    fn abort() -> Self {
        Self::stand_pat(Score::DRAW)
    }

    fn score(&self) -> Score {
        self.score
    }

    fn push_pair(&mut self, response: Pos, attack: Pos) {
        self.sequence.push(response);
        self.sequence.push(attack);
    }
}

pub fn quiescence_search<const R: RuleKind, const VCT: bool>(
    td: &mut ThreadData<R, impl ThreadType, impl Evaluator<R>>,
    max_ply: u8,
    state: &mut GameState<R>,
    alpha: Score, beta: Score,
    static_eval: Score,
    is_pv: bool,
) -> Score {
    let indexes = state.board.patterns.indexes[state.board.player_color];

    let empty_fours = !indexes.has_any_four();

    if (!VCT && empty_fours)
        || (VCT && empty_fours && indexes.open_threes.is_empty())
    {
        return static_eval;
    }

    let recent_player_action = state.history.last_action_pair()[0].unwrap_or(pos::CENTER);

    let mut endgame_moves = generate_endgame_moves::<R, VCT>(&state.board, 8, recent_player_action);

    if endgame_moves.is_empty() {
        return static_eval;
    }

    endgame_moves.sort_moves(&state.board, recent_player_action);
    endgame_moves.init();

    if VCT {
        todo!()
    } else {
        let context = EndgameContext { beta, is_pv };
        let history = state.history;

        let score = match state.board.player_color {
            Color::Black => try_vcf::<R, { Color::Black }, 5, _, Score>(
                td, &context, max_ply, state, endgame_moves, alpha, static_eval, 0,
            ),
            Color::White => try_vcf::<R, { Color::White }, 5, _, Score>(
                td, &context, max_ply, state, endgame_moves, alpha, static_eval, 0,
            ),
        };

        state.history = history;
        score
    }
}

pub fn endgame_proof<const R: RuleKind, const VCT: bool>(
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
        let context = EndgameContext { beta: Score::INF, is_pv: true };

        let mut state = *state;
        let static_eval = td.evaluator.eval_value(&state);

        let mut proof = match state.board.player_color {
            Color::Black => try_vcf::<R, { Color::Black }, 8, _, SequenceProof>(
                td, &context, pos::U8_BOARD_SIZE, &mut state, endgame_moves, Score::NEG_INF, static_eval, 0,
            ),
            Color::White => try_vcf::<R, { Color::White }, 8, _, SequenceProof>(
                td, &context, pos::U8_BOARD_SIZE, &mut state, endgame_moves, Score::NEG_INF, static_eval, 0,
            ),
        };

        if !proof.score.is_win() || proof.sequence.is_empty() {
            None
        } else {
            proof.sequence.reverse();
            Some(proof.sequence)
        }
    }
}

fn try_vcf<const R: RuleKind, const C: Color, const DW: u8, TH: ThreadType, Pf: EndgameProof>(
    td: &mut ThreadData<R, TH, impl Evaluator<R>>,
    context: &EndgameContext,
    vcf_pair_depth_left: u8,
    state: &mut GameState<R>,
    mut vcf_moves: EndgameMovesUnchecked,
    mut alpha: Score,
    static_eval: Score,
    vcf_ply: usize,
) -> Pf {
    let ply = td.ply + vcf_ply;

    if td.is_aborted() {
        return Pf::abort();
    }

    let mut best_proof = Pf::stand_pat(static_eval);
    let mut best_score = best_proof.score();

    alpha = alpha.max(best_score).max(Score::lose_in(ply));

    if alpha >= context.beta || alpha >= Score::win_in(ply + 1) {
        return best_proof;
    }

    while let Some(four_pos) = vcf_moves.next() {
        if TH::IS_MAIN
            && td.should_check_limit()
            && td.search_limit_exceeded()
        {
            td.set_aborted();
        }

        if td.is_aborted() {
            return Pf::abort();
        }

        let player_pattern = state.board.patterns.field[C][four_pos.idx_usize()];

        if R == RuleKind::Renju && C == Color::Black && state.board.patterns.is_forbidden(four_pos) {
            continue;
        }

        if player_pattern.has_open_four() {
            let proof = Pf::entry(four_pos, ply + 3);

            tt_store_vcf_win(&td.tt, state.board.hash_key, four_pos, static_eval, proof.score(), ply, context.is_pv);

            return proof;
        }

        let Some(child_depth) = vcf_pair_depth_left.checked_sub(1) else {
            continue;
        };

        td.batch_counter.increment();
        let recovery_state = state.recovery_state();
        let artifact = state.play_mut(four_pos);
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
            } {
                break 'candidate Pf::stand_pat(static_eval);
            }

            if (C == Color::White && response_is_forbidden)
                || (response_four_count == 0 && player_pattern.has_open_three())
            {
                break 'candidate Pf::entry(four_pos, ply + 1);
            }

            if !Pf::SEQUENCE && let Some(TTEntryBucketProbe { entry, .. }) = td.tt.probe(tt_key) {
                let tt_score = MaybeScore::from(entry.score as i32);

                // tt cutoff
                if tt_score.is_some() && tt_score.unwrap().is_win()
                    && matches!(entry.tt_flag.maybe_score_kind(), Some(ScoreKind::Exact | ScoreKind::LowerBound))
                {
                    let score = transposition_table::decode_mate_distance(tt_score.unwrap(), ply + 2);
                    let mate_ply = (Score::MATE.value() - score.value()) as usize;
                    break 'candidate Pf::entry(four_pos, mate_ply);
                }
            }

            td.batch_counter.increment();
            let response_eval = td.evaluator.eval_value(state);
            let response_recovery_state = state.recovery_state();
            let artifact = state.play_mut(response_pos);
            td.evaluator.play(&state.board, artifact, response_pos.into());

            let child_moves = if !state.board.patterns.indexes[C].has_any_four() {
                None // cold branch stand-pat
            } else if response_four_count != 0 {
                let response_move = state.board.patterns.five_pos[!C].unwrap();

                if !state.board.patterns.field[C][response_move.idx_usize()].has_any_four()
                    || (R == RuleKind::Renju && C == Color::Black && state.board.patterns.is_forbidden(response_move))
                {
                    None // unable to continue with four stand-pat
                } else {
                    Some(EndgameMovesUnchecked::unit(response_move))
                }
            } else {
                Some(generate_endgame_moves::<R, false>(&state.board, DW, four_pos))
            };

            let child_eval = td.evaluator.eval_value(state);

            let mut proof = if let Some(mut moves) = child_moves {
                moves.init();

                try_vcf::<R, C, DW, TH, Pf>(
                    td, context, child_depth, state, moves, alpha, child_eval, vcf_ply + 2,
                )
            } else {
                Pf::stand_pat(child_eval)
            };

            let artifact = state.undo_mut(response_recovery_state);
            td.evaluator.undo(&state.board, artifact, response_pos.into());

            if td.is_aborted() {
                break 'candidate Pf::abort();
            }

            if proof.score().is_win() {
                tt_store_vcf_lose(&td.tt, state.board.hash_key, response_pos, response_eval, -proof.score(), ply + 1, context.is_pv);
                proof.push_pair(response_pos, four_pos);
            }

            proof
        };

        let artifact = state.undo_mut(recovery_state);
        td.evaluator.undo(&state.board, artifact, four_pos.into());

        if td.is_aborted() {
            return Pf::abort();
        }

        let score = proof.score();

        if score.is_win() {
            tt_store_vcf_win(&td.tt, state.board.hash_key, four_pos, static_eval, score, ply, context.is_pv);
            return proof;
        }

        if score > best_score {
            best_score = score;
            best_proof = proof;
            alpha = alpha.max(score);
        }

        if alpha >= context.beta {
            return best_proof;
        }
    }

    if td.is_aborted() {
        return Pf::abort();
    }

    best_proof
}

fn tt_store_vcf_win(
    tt: &TTView,
    hash_key: HashKey,
    four_pos: Pos,
    eval: Score,
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
        eval.into(),
        transposition_table::encode_mate_distance(score, ply).into(),
        is_pv,
    );
}

#[inline]
fn tt_store_vcf_lose(
    tt: &TTView,
    hash_key: HashKey,
    response_pos: Pos,
    eval: Score,
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
        eval.into(),
        transposition_table::encode_mate_distance(score, ply).into(),
        is_pv,
    );
}
