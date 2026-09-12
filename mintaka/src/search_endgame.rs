use crate::eval::evaluator::Evaluator;
use crate::game_state::GameState;
use crate::memo::transposition_table;
use crate::memo::tt_entry::{ScoreKind, TTEntry, TTEntryBucketProbe};
use crate::movegen::move_generator::{generate_endgame_moves, generate_full_endgame_moves};
use crate::movegen::move_list::{EndgameMoveEntry, EndgameMoveList};
use crate::principal_variation::PrincipalVariation;
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use crate::utils::depth;
use crate::utils::depth::Depth;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::score::{MaybeScore, Score};
#[cfg(feature = "typeshare")]
use typeshare::typeshare;
use crate::config::Config;

pub fn find_immediate_win<const R: RuleKind>(config: &Config, state: &GameState<R>, ply: usize) -> (MaybeScore, MaybePos) {
    fn mate_in_ply<const R: RuleKind>(config: &Config, state: &GameState<R>, mate_in: usize, score: Score, pos: MaybePos) -> (MaybeScore, MaybePos) {
        let score = if mate_in > config.draw_condition
            .map(|draw_in|
                (draw_in as usize).saturating_sub(state.history.len())
            )
            .unwrap_or(usize::MAX)
            .min(pos::BOARD_SIZE - state.board.stones as usize)
        {
            Score::DRAW.into()
        } else {
            score.into()
        };

        (score, pos)
    }

    if let Some(pos) = state.board.patterns.five_pos[state.board.player_color].ok() {
        return mate_in_ply(config, state, 1, Score::win_in(ply + 1), pos.into())
    }

    if let Some(pos) = state.board.patterns.five_pos[!state.board.player_color].ok() {
        if !state.board.is_legal_move(pos) {
            return mate_in_ply(config, state, 2, Score::lose_in(ply + 2), MaybePos::NONE)
        }

        if 1 < state.board.patterns.field[!state.board.player_color].iter()
            .filter(|pattern| pattern.has_five())
            .count()
        {
            return mate_in_ply(config, state, 2, Score::lose_in(ply + 2), pos.into())
        }

        return (MaybeScore::NONE, pos.into())
    }

    if let Some(pos) = state.board.patterns.effective_fork_four_field(state.board.player_color).first_pos() {
        return mate_in_ply(config, state, 3, Score::win_in(ply + 3), pos.into());
    }

    (MaybeScore::NONE, MaybePos::NONE)
}

#[cfg_attr(feature = "typeshare", typeshare(serialized_as = "String"))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(std::marker::ConstParamTy, PartialEq, Eq)]
pub enum ThreatSearchKind {
    VCF, VCT, Forced,
}

pub fn min_endgame_stones<const T: ThreatSearchKind>() -> u8 {
    match T {
        ThreatSearchKind::VCF => 9,
        ThreatSearchKind::VCT => 6,
        ThreatSearchKind::Forced => 6,
    }
}

struct EndgameContext {
    min_depth: Depth,
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
    pv: &mut PrincipalVariation,
    depth_left: Depth,
    state: &mut GameState<R>,
    alpha: Score, beta: Score,
    is_pv: bool,
) -> Score {
    let context = EndgameContext {
        min_depth: Depth::ZERO - td.config.max_quiescence_depth.unwrap_or(Depth::PLY_LIMIT)
            .min(Depth::PLY_LIMIT - td.ply as i32),
        beta, is_pv,
    };

    match T {
        ThreatSearchKind::VCF => {
            match state.board.player_color {
                Color::Black => try_vcf::<R, { Color::Black }, _, Score>(
                    td, &context, pv, state, depth_left, td.ply, alpha,
                ),
                Color::White => try_vcf::<R, { Color::White }, _, Score>(
                    td, &context, pv, state, depth_left, td.ply, alpha,
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
    let context = EndgameContext {
        min_depth: Depth::ZERO - td.config.max_quiescence_depth.unwrap_or(Depth::PLY_LIMIT)
            .min(Depth::PLY_LIMIT - td.ply as i32),
        beta: Score::INF, is_pv: true,
    };

    let mut pv = PrincipalVariation::EMPTY;
    let proof = match T {
        ThreatSearchKind::VCF => {
            match state.board.player_color {
                Color::Black => try_vcf::<R, { Color::Black }, _, SequenceProof>(
                    td, &context, &mut pv, state, Depth::ZERO, td.ply, Score::NEG_INF,
                ),
                Color::White => try_vcf::<R, { Color::White }, _, SequenceProof>(
                    td, &context, &mut pv, state, Depth::ZERO, td.ply, Score::NEG_INF,
                ),
            }
        },
        ThreatSearchKind::VCT => todo!(),
        ThreatSearchKind::Forced => todo!(),
    };

    if td.is_aborted() || !proof.score.is_win() || proof.sequence.is_empty() {
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
    pv: &mut PrincipalVariation,
    state: &mut GameState<R>,
    depth_left: Depth,
    ply: usize,
    mut alpha: Score,
) -> Pf {
    pv.clear();
    if TH::IS_MAIN
        && td.should_check_limit()
        && td.search_limit_exceeded()
    {
        td.set_aborted();
    }

    if td.is_aborted() {
        return Pf::abort();
    }

    td.batch_counter.increment();

    td.selective_depth = td.selective_depth.max(Depth::from_i32(ply as i32));

    if state.board.stones as usize == pos::BOARD_SIZE
        || td.config.draw_condition.is_some_and(|draw_in| state.len() as u32 >= draw_in)
    {
        return Pf::stand_pat(Score::DRAW);
    }

    let (immediate_score, forced_move) = find_immediate_win(&td.config, state, ply);

    if let Some(score) = immediate_score.ok() {
        if score == Score::DRAW {
            return Pf::stand_pat(Score::DRAW);
        }

        let mate_ply = (Score::MATE.value() - score.value().abs()) as usize;

        td.tt.store(
            state.board.hash_key,
            forced_move,
            Depth::ZERO,
            TTEntry::QUIESCENCE_PROVEN_DEPTH,
            Some(if score.is_win() { ScoreKind::LowerBound } else { ScoreKind::UpperBound }),
            MaybeScore::NONE,
            transposition_table::encode_mate_distance(score, ply).into(),
            context.is_pv,
        );

        if context.is_pv && ply < depth::MAX_PLY && let Some(pos) = forced_move.ok() {
            pv.set(pos);
        }

        if score.is_win() {
            return Pf::entry(forced_move.unwrap(), mate_ply);
        }

        return Pf::stand_pat(score);
    }

    let mut quiescence_depth = (depth_left - context.min_depth).value().max(0) as u8 + 1;
    let mut beta = context.beta;

    if !Pf::COMPLETE_PROOF {
        alpha = alpha.max(Score::lose_in(ply));
        beta = beta.min(Score::win_in(ply + 1));

        if alpha >= beta {
            return Pf::stand_pat(alpha);
        }
    }

    let mut tt_move = MaybePos::NONE;
    let mut evaluator_eval = MaybeScore::NONE;
    let mut tt_pv = context.is_pv;

    if let Some(TTEntryBucketProbe { entry, .. }) = td.tt.probe(state.board.hash_key) {
        tt_move = entry.best_move;
        evaluator_eval = MaybeScore::from(entry.eval as i32);
        tt_pv |= entry.tt_flag.is_pv();

        let tt_score = MaybeScore::from(entry.score as i32);
        let score_kind = entry.tt_flag.maybe_score_kind();

        if entry.depth == 0 && entry.quiescence_depth >= quiescence_depth {
            if Pf::COMPLETE_PROOF {
                if entry.quiescence_depth < TTEntry::QUIESCENCE_PROVEN_DEPTH
                    && score_kind.is_none() && tt_score.is_none()
                {
                    return Pf::abort();
                }
            } else if !context.is_pv && tt_score.is_some() {
                let score = transposition_table::decode_mate_distance(tt_score.unwrap(), ply);

                if match score_kind {
                    Some(ScoreKind::LowerBound) => score >= beta,
                    Some(ScoreKind::UpperBound) => score <= alpha,
                    Some(ScoreKind::Exact) => true,
                    None => false,
                } {
                    return Pf::stand_pat(score);
                }
            }
        }
    }

    let static_eval = if Pf::COMPLETE_PROOF {
        Score::DRAW
    } else {
        evaluator_eval.unwrap_or_else(|| 
            td.evaluator.eval_value(state)
        )
    };

    let original_alpha = alpha;
    let mut best_move = MaybePos::NONE;
    let mut best_proof = if !Pf::COMPLETE_PROOF && forced_move.is_some() {
        Pf::stand_pat(Score::NEG_INF)
    } else {
        Pf::stand_pat(static_eval)
    };
    let mut best_score = best_proof.score();

    'search: {
        if ply >= depth::MAX_PLY {
            best_proof = Pf::stand_pat(if forced_move.is_some() { Score::DRAW } else { static_eval });
            best_score = best_proof.score();
            break 'search;
        }

        if !Pf::COMPLETE_PROOF && forced_move.is_none() {
            if best_score >= beta {
                quiescence_depth = 1;
                break 'search;
            }

            alpha = alpha.max(best_score);
        }

        if depth_left <= context.min_depth && (Pf::COMPLETE_PROOF || forced_move.is_none()) {
            break 'search;
        }

        let mut moves = if let Some(pos) = forced_move.ok() {
            EndgameMoveList::unit(EndgameMoveEntry { pos, score: 0 })
        } else if Pf::COMPLETE_PROOF {
            generate_full_endgame_moves::<R, { ThreatSearchKind::VCF }>(state)
        } else {
            generate_endgame_moves::<R, { ThreatSearchKind::VCF }>(
                td, state,
                state.history.previous_action().unwrap(),
            )
        };

        if let Some(tt_move) = tt_move.ok() {
            for entry in moves.iter_mut() {
                if entry.pos == tt_move {
                    entry.score = i16::MAX;
                    break;
                }
            }
        }

        let mut child_pv = PrincipalVariation::EMPTY;

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

            if !state.board.is_legal_move(four_pos) {
                continue;
            }

            let player_pattern = state.board.patterns.field[C][four_pos.idx_usize()];

            if Pf::COMPLETE_PROOF && !player_pattern.has_any_four() {
                continue;
            }

            let mut searched_response = MaybePos::NONE;

            let recovery_state = state.recovery_state();
            let artifact = state.play_mut(four_pos);
            td.evaluator.play(&state.board, artifact, four_pos.into());
            td.batch_counter.increment();

            if !Pf::COMPLETE_PROOF {
                td.selective_depth = td.selective_depth.max(Depth::from_i32(ply as i32 + 1));
            }

            let proof = 'candidate: {
                if state.board.stones as usize == pos::BOARD_SIZE
                    || td.config.draw_condition.is_some_and(|draw_in| state.len() as u32 >= draw_in)
                {
                    break 'candidate Pf::stand_pat(Score::DRAW);
                }

                let (response_score, response_pos) = find_immediate_win(&td.config, state, ply + 1);

                if let Some(score) = response_score.ok() {
                    let mate_ply = (Score::MATE.value() - score.value().abs()) as usize;

                    if score == Score::DRAW {
                        break 'candidate Pf::stand_pat(Score::DRAW);
                    }

                    td.tt.store(
                        state.board.hash_key,
                        response_pos,
                        Depth::ZERO,
                        TTEntry::QUIESCENCE_PROVEN_DEPTH,
                        Some(if score.is_win() { ScoreKind::LowerBound } else { ScoreKind::UpperBound }),
                        MaybeScore::NONE,
                        transposition_table::encode_mate_distance(score, ply + 1).into(),
                        context.is_pv,
                    );

                    break 'candidate if score.is_lose() {
                        Pf::entry(four_pos, mate_ply)
                    } else {
                        Pf::stand_pat(-score)
                    };
                }

                let Some(response_pos) = response_pos.ok() else {
                    break 'candidate Pf::stand_pat(-td.evaluator.eval_value(state));
                };

                if ply + 2 > depth::MAX_PLY {
                    break 'candidate Pf::stand_pat(Score::DRAW);
                }

                td.tt.prefetch(state.board.hash_key.set(!C, response_pos));

                let response_recovery_state = state.recovery_state();
                let artifact = state.play_mut(response_pos);
                td.evaluator.play(&state.board, artifact, response_pos.into());

                let mut proof = try_vcf::<R, C, TH, Pf>(
                    td, context, &mut child_pv, state, depth_left - 2, ply + 2, alpha,
                );

                searched_response = response_pos.into();

                let three_four_fork_proof = Pf::COMPLETE_PROOF
                    && player_pattern.has_open_three()
                    && proof.score() == Score::win_in(ply + 5)
                    && state.board.patterns.five_pos[!C].is_none()
                    && !state.board.patterns.effective_fork_four_field(C).is_empty();

                let artifact = state.undo_mut(response_recovery_state);
                td.evaluator.undo(&state.board, artifact, response_pos.into());

                if td.is_aborted() {
                    break 'candidate Pf::abort();
                }

                if (Pf::COMPLETE_PROOF || proof.score() > alpha) && proof.score().is_win() {
                    let response_eval = td.evaluator.eval_value(state).into();

                    td.tt.store(
                        state.board.hash_key,
                        response_pos.into(),
                        Depth::ZERO,
                        TTEntry::QUIESCENCE_PROVEN_DEPTH,
                        Some(ScoreKind::UpperBound),
                        response_eval,
                        transposition_table::encode_mate_distance(-proof.score(), ply + 1).into(),
                        context.is_pv,
                    );
                }

                if Pf::COMPLETE_PROOF && proof.score().is_win() {
                    if three_four_fork_proof {
                        proof = Pf::entry(four_pos, ply + 5);
                    } else {
                        proof.push_pair(response_pos, four_pos);
                    }
                }

                proof
            };

            let artifact = state.undo_mut(recovery_state);
            td.evaluator.undo(&state.board, artifact, four_pos.into());

            if td.is_aborted() {
                return Pf::abort();
            }

            let score = proof.score();
            if score > best_score {
                best_score = score;
                best_proof = proof;

                if score > alpha {
                    best_move = four_pos.into();
                    alpha = score;

                    if context.is_pv {
                        if let Some(response) = searched_response.ok() {
                            pv.update_pair(four_pos, response, &child_pv);
                        } else {
                            pv.set(four_pos);
                        }
                    }
                }
            }

            if !Pf::COMPLETE_PROOF && alpha >= beta {
                break;
            }

            if Pf::COMPLETE_PROOF && best_score.is_win() {
                break;
            }
        }
    }

    if td.is_aborted() {
        return Pf::abort();
    }

    let mut score_kind = if Pf::COMPLETE_PROOF {
        best_score.is_win().then_some(ScoreKind::LowerBound)
    } else if best_score >= beta {
        Some(ScoreKind::LowerBound)
    } else if best_score > original_alpha {
        Some(ScoreKind::Exact)
    } else {
        Some(ScoreKind::UpperBound)
    };

    if best_score.is_win() && matches!(score_kind, Some(ScoreKind::LowerBound | ScoreKind::Exact))
        || best_score.is_lose() && matches!(score_kind, Some(ScoreKind::UpperBound | ScoreKind::Exact))
    {
        quiescence_depth = TTEntry::QUIESCENCE_PROVEN_DEPTH;
        score_kind = Some(if best_score.is_win() { ScoreKind::LowerBound } else { ScoreKind::UpperBound });
    }

    td.tt.store(
        state.board.hash_key,
        best_move,
        Depth::ZERO,
        quiescence_depth,
        score_kind,
        evaluator_eval,
        if score_kind.is_some() {
            transposition_table::encode_mate_distance(best_score, ply).into()
        } else {
            MaybeScore::NONE
        },
        tt_pv,
    );

    best_proof
}
