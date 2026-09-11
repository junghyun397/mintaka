use crate::eval::evaluator::Evaluator;
use crate::game_state::GameState;
use crate::memo::transposition_table;
use crate::memo::transposition_table::TTView;
use crate::memo::tt_entry::{ScoreKind, TTEntry, TTEntryBucketProbe};
use crate::movegen::move_generator::{generate_endgame_moves, generate_full_endgame_moves};
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use crate::utils::depth::Depth;
use rusty_renju::hash_key::HashKey;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::score::{MaybeScore, Score};
#[cfg(feature = "typeshare")]
use typeshare::typeshare;
use crate::movegen::move_list::{EndgameMoveEntry, EndgameMoveList};

#[cfg_attr(feature = "typeshare", typeshare(serialized_as = "String"))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(std::marker::ConstParamTy, PartialEq, Eq)]
pub enum ThreatSearchKind {
    VCF, VCT, Forced,
}

struct EndgameContext {
    max_depth: Depth,
    beta: Score,
    is_pv: bool,
}

trait EndgameProof {
    const COMPLETE_PROOF: bool;

    fn entry(four_pos: Pos, ply: usize) -> Self;

    fn stand_pat(eval: Score) -> Self;

    fn abort() -> Self;

    fn score(&self) -> Score;

    fn push_pair(&mut self, response: Pos, attack: Pos);
}

impl EndgameProof for Score {
    const COMPLETE_PROOF: bool = false;

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
    const COMPLETE_PROOF: bool = true;

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

pub fn quiescence_search<const R: RuleKind, const T: ThreatSearchKind>(
    td: &mut ThreadData<R, impl ThreadType, impl Evaluator<R>>,
    max_quiescence_depth: Option<Depth>,
    state: &mut GameState<R>,
    alpha: Score, beta: Score,
    static_eval: Score,
    is_pv: bool,
) -> Score {
    let indexes = &state.board.patterns.indexes[state.board.player_color];

    if match T {
        ThreatSearchKind::VCF => !indexes.has_any_four(),
        ThreatSearchKind::VCT => indexes.open_threes.is_empty() || !indexes.has_any_four(),
        ThreatSearchKind::Forced => todo!()
    } {
        return static_eval;
    }

    let recent_player_action = state.history.last_action_pair()[0].unwrap_or(pos::CENTER);

    let moves = generate_endgame_moves::<R, T>(td, state, recent_player_action);

    if moves.is_empty() {
        return static_eval;
    }

    let context = EndgameContext {
        max_depth: max_quiescence_depth.unwrap_or(Depth::BOARD_LIMIT),
        beta, is_pv,
    };

    match T {
        ThreatSearchKind::VCF => {
            match state.board.player_color {
                Color::Black => try_vcf::<R, { Color::Black }, _, Score>(
                    td, &context, state, moves, context.max_depth, alpha, static_eval,
                ),
                Color::White => try_vcf::<R, { Color::White }, _, Score>(
                    td, &context, state, moves, context.max_depth, alpha, static_eval,
                ),
            }
        },
        ThreatSearchKind::VCT => todo!(),
        ThreatSearchKind::Forced => todo!()
    }
}

pub fn endgame_proof<const R: RuleKind, const T: ThreatSearchKind>(
    td: &mut ThreadData<R, impl ThreadType, impl Evaluator<R>>,
    state: &mut GameState<R>,
) -> Option<Vec<Pos>> {
    let mut moves = generate_full_endgame_moves::<R, T>(state);

    if moves.is_empty() {
        return None;
    }

    let context = EndgameContext {
        max_depth: td.config.max_quiescence_depth.unwrap_or(Depth::BOARD_LIMIT),
        beta: Score::INF, is_pv: true,
    };

    let proof = match T {
        ThreatSearchKind::VCF => {
            match state.board.player_color {
                Color::Black => try_vcf::<R, { Color::Black }, _, SequenceProof>(
                    td, &context, state, moves, context.max_depth, Score::NEG_INF, Score::NEG_INF,
                ),
                Color::White => try_vcf::<R, { Color::White }, _, SequenceProof>(
                    td, &context, state, moves, context.max_depth, Score::NEG_INF, Score::NEG_INF,
                ),
            }
        },
        ThreatSearchKind::VCT => todo!(),
        ThreatSearchKind::Forced => todo!(),
    };

    if !proof.score.is_win() || proof.sequence.is_empty() {
        None
    } else {
        let mut sequence = proof.sequence;
        sequence.reverse();
        Some(sequence)
    }
}

fn try_vcf<const R: RuleKind, const C: Color, TH: ThreadType, Pf: EndgameProof>(
    td: &mut ThreadData<R, TH, impl Evaluator<R>>,
    context: &EndgameContext,
    state: &mut GameState<R>,
    mut moves: EndgameMoveList,
    depth_left: Depth,
    mut alpha: Score,
    static_eval: Score,
) -> Pf {
    let ply = td.ply;

    if td.is_aborted() {
        return Pf::abort();
    }

    let mut best_proof = Pf::stand_pat(static_eval);
    let mut best_score = best_proof.score();

    alpha = alpha.max(best_score).max(Score::lose_in(ply));

    if alpha >= context.beta || alpha >= Score::win_in(ply + 1) {
        return best_proof;
    }

    while let Some(EndgameMoveEntry { pos: four_pos, .. }) = moves.consume_best() {
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

        if Depth::ZERO >= depth_left {
            continue;
        }

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

            if let Some(TTEntryBucketProbe { entry, .. }) = td.tt.probe(tt_key) {
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

            let child_moves = 'movegen: {
                if !state.board.patterns.indexes[C].has_any_four() {
                    break 'movegen None;
                }

                if response_four_count == 0 {
                    break 'movegen Some(if Pf::COMPLETE_PROOF {
                        generate_full_endgame_moves::<R, { ThreatSearchKind::VCF }>(state)
                    } else {
                        generate_endgame_moves::<R, { ThreatSearchKind::VCF }>(td, state, four_pos)
                    })
                }

                let response_move = state.board.patterns.five_pos[!C].unwrap();

                if state.board.patterns.field[C][response_move.idx_usize()].has_any_four()
                    && (R != RuleKind::Renju || C != Color::Black || !state.board.patterns.is_forbidden(response_move))
                {
                    break 'movegen Some(EndgameMoveList::unit(EndgameMoveEntry { pos: response_move, score: 0 }))
                }

                None
            };

            let child_eval = td.evaluator.eval_value(state);

            let mut proof = if let Some(moves) = child_moves {
                try_vcf::<R, C, TH, Pf>(
                    td, context, state, moves, depth_left - 2, alpha, child_eval,
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

        if !Pf::COMPLETE_PROOF && alpha >= context.beta {
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
        TTEntry::QUIESCENCE_PROVEN_DEPTH,
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
        TTEntry::QUIESCENCE_PROVEN_DEPTH,
        Some(ScoreKind::UpperBound),
        eval.into(),
        transposition_table::encode_mate_distance(score, ply).into(),
        is_pv,
    );
}
