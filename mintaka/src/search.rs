use crate::eval::evaluator::Evaluator;
use crate::game_state::GameState;
use crate::memo::transposition_table;
use crate::memo::tt_entry::{ScoreKind, TTEntry, TTEntryBucketProbe};
use crate::movegen::move_generator;
use crate::movegen::move_list::MainMoveEntry;
use crate::movegen::move_picker::{MovePicker, ThreatKind};
use crate::params;
use crate::principal_variation::PrincipalVariation;
use crate::protocol::response::Response;
use crate::search_endgame::{find_immediate_win, min_endgame_stones, quiescence_search, ThreatSearchKind};
use crate::thread_data::{SearchFrame, ThreadData};
use crate::thread_type::ThreadType;
use crate::utils::depth::Depth;
use rusty_renju::bitfield::Bitfield;
use rusty_renju::const_for;
use rusty_renju::notation::pos::{self, MaybePos};
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::score::{MaybeScore, Score};
use crate::utils::depth;

trait NodeType {
    const IS_ROOT: bool;
    const IS_PV: bool;

    type NextType: NodeType;
}

struct RootNode; impl NodeType for RootNode {
    const IS_ROOT: bool = true;
    const IS_PV: bool = true;
    type NextType = PVNode;
}

struct PVNode; impl NodeType for PVNode {
    const IS_ROOT: bool = false;
    const IS_PV: bool = true;
    type NextType = Self;
}

struct OffPVNode; impl NodeType for OffPVNode {
    const IS_ROOT: bool = false;
    const IS_PV: bool = false;
    type NextType = Self;
}

pub struct SearchResult {
    pub score: Score,
    pub pv: PrincipalVariation,
    pub selective_depth: Depth,
}

impl SearchResult {
    pub fn best_move(&self) -> MaybePos {
        self.pv.first()
    }
}

pub fn iterative_deepening<const R: RuleKind, TH: ThreadType>(
    td: &mut ThreadData<R, TH, impl Evaluator<R>>,
    mut state: GameState<R>,
) -> SearchResult {
    let position_hash = state.board.hash_key;

    let mut pv = PrincipalVariation::EMPTY;
    let mut result = SearchResult {
        score: Score::DRAW,
        pv: PrincipalVariation::EMPTY,
        selective_depth: Depth::ZERO,
    };

    let mut mate_count = 0;
    let mut best_move_changes = 0;

    let starting_depth = Depth::from_i32(td.tid as i32 % 10 + 1);
    'iterative_deepening: for depth in starting_depth.value() ..= td.config.max_depth().value() {
        let depth = Depth::from_i32(depth);

        let iter_score = if depth < Depth::from_i32(5) {
            pvs::<R, TH, RootNode>(td, &mut pv, &mut state, depth, Score::NEG_INF, Score::INF, false)
        } else {
            aspiration::<R, TH>(&mut pv, td, &mut state, depth, result.score)
        };

        if td.is_aborted() {
            break 'iterative_deepening;
        }

        if result.best_move() != pv.first() {
            best_move_changes += 1;
        }

        result = SearchResult {
            score: iter_score,
            pv,
            selective_depth: td.selective_depth,
        };

        if TH::IS_MAIN {
            td.thread_type.make_response(Response::Status {
                hash: position_hash,
                best_move: result.best_move(),
                score: result.score,
                pv: result.pv,
                total_nodes_in_1k: td.batch_counter.count_global_in_1k(),
                time_elapsed: td.thread_type.time_manager().elapsed(),
                selective_depth: result.selective_depth,
            })
        }

        if iter_score.is_mate() {
            mate_count += 1;

            if depth - starting_depth > Depth::from_i32(10)
                && mate_count > 4
            {
                break 'iterative_deepening;
            }
        } else {
            mate_count = 0;
        }
        
        if TH::IS_MAIN {
            let best_move_search_share = result.best_move().ok()
                .map(|pos|
                    td.root_moves_in_1k[pos.idx_usize()] as f64 / td.batch_counter.count_local_in_1k().max(1) as f64
                )
                .unwrap_or(0.0);

            td.thread_type.time_manager_mut().update_each_depth(
                td.singular_root,
                best_move_changes,
                best_move_search_share,
            )
        }

        if TH::IS_MAIN
            && td.thread_type.time_manager().is_soft_limit_reached()
        {
            break 'iterative_deepening;
        }

        best_move_changes = 0;

        td.singular_root = false;
    }

    if TH::IS_MAIN {
        td.set_aborted();
    }

    result
}

fn aspiration<const R: RuleKind, TH: ThreadType>(
    pv: &mut PrincipalVariation,
    td: &mut ThreadData<R, TH, impl Evaluator<R>>,
    state: &mut GameState<R>,
    max_depth: Depth,
    prev_score: Score,
) -> Score {
    let mut depth = max_depth;

    let mut delta = params::ASPIRATION_DELTA_BASE + prev_score.value().pow(2) / params::ASPIRATION_DELTA_DIV;
    let mut alpha = prev_score - delta;
    let mut beta = prev_score + delta;

    loop {
        let score = pvs::<R, TH, RootNode>(td, pv, state, depth, alpha, beta, false);

        if td.is_aborted() {
            return Score::DRAW;
        }

        if score <= alpha { // fail-low
            beta = (alpha + beta) / 2;
            alpha = score - delta;
            depth = max_depth;

            if TH::IS_MAIN {
                td.thread_type.time_manager_mut().update_fail_low();
            }
        } else if score >= beta { // fail-high
            beta = score + delta;
            depth = max_depth;
        } else { // exact
            return score;
        }

        delta *= 2;
    }
}

fn pvs<const R: RuleKind, TH: ThreadType, NT: NodeType>(
    td: &mut ThreadData<R, TH, impl Evaluator<R>>,
    pv: &mut PrincipalVariation,
    state: &mut GameState<R>,
    depth_left: Depth,
    mut alpha: Score,
    mut beta: Score,
    cut_node: bool,
) -> Score {
    pv.clear();

    if TH::IS_MAIN
        && td.should_check_limit()
        && td.search_limit_exceeded()
    {
        td.set_aborted();
        return Score::DRAW;
    }

    if td.is_aborted() {
        return Score::DRAW;
    }

    if state.board.stones as usize == pos::BOARD_SIZE
        || td.config.draw_condition.is_some_and(|draw_in| state.len() as u32 >= draw_in)
    {
        return Score::DRAW;
    }

    if td.ply >= depth::MAX_PLY {
        return td.evaluator.eval_value(state);
    }

    td.batch_counter.increment();

    td.selective_depth = td.selective_depth.max((td.ply as i32).into());

    let mut child_pv = PrincipalVariation::EMPTY;

    {
        let (score, pos) = find_immediate_win(&td.config, state, td.ply);

        if score.is_some() { // immediate win or lose
            if NT::IS_ROOT {
                td.singular_root = true;
            }

            if NT::IS_PV && let Some(pos) = pos.ok() {
                pv.set(pos);
            }

            return score.unwrap();
        }

        if let Some(pos) = pos.ok() { // defend immediate win
            if NT::IS_ROOT {
                td.singular_root = true;
            }

            let parent_eval = if td.ply > 0 {
                -td.ss[td.ply - 1].static_eval
            } else {
                Score::DRAW
            };

            td.ss[td.ply] = SearchFrame {
                pos: pos.into(),
                static_eval: parent_eval,
                evaluator_eval: MaybeScore::NONE,
                on_pv: NT::IS_PV,
                recovery_state: state.recovery_state(),
                searching: MaybePos::NONE,
            };

            td.push_ply(pos);
            let artifact = state.play_mut(pos);
            td.evaluator.play(&state.board, artifact, pos.into());

            // no depth reduction for forced response
            let score = -pvs::<R, TH, NT::NextType>(td, &mut child_pv, state, depth_left, -beta, -alpha, cut_node);

            td.pop_ply();
            let artifact = state.undo_mut(td.ss[td.ply].recovery_state);
            td.evaluator.undo(&state.board, artifact, pos.into());

            if td.is_aborted() {
                return Score::DRAW;
            }

            if NT::IS_PV {
                pv.update(pos, &child_pv);
            }

            return score;
        }
    }

    if depth_left <= Depth::ZERO {
        return if state.board.stones >= min_endgame_stones::<{ ThreatSearchKind::VCF }>() {
            quiescence_search::<R, { ThreatSearchKind::VCF }>(
                td, pv, depth_left.min(Depth::ZERO), state, alpha, beta, NT::IS_PV,
            )
        } else {
            td.evaluator.eval_value(state)
        }
    }

    if !NT::IS_ROOT {
        alpha = alpha.max(Score::lose_in(td.ply));
        beta = beta.min(Score::win_in(td.ply));
        if alpha >= beta { // mate distance pruning
            return alpha;
        }
    }

    let threat_kind = 'threat_kind: {
        let fork_four_field = state.board.patterns.effective_fork_four_field(!state.board.player_color);

        if !fork_four_field.is_empty() {
            break 'threat_kind Some(ThreatKind::ForkFour(fork_four_field));
        }

        let fork_three_four_field = state.board.patterns.effective_fork_three_four_field(!state.board.player_color);

        if !fork_three_four_field.is_empty() {
            break 'threat_kind Some(ThreatKind::ForkThreeFour(
                fork_three_four_field
                    | state.board.patterns.indexes[!state.board.player_color].open_threes
            ));
        }

        None
    };

    let evaluator_eval: MaybeScore;
    let tt_move: MaybePos;
    let tt_pv: bool;

    let tt_entry = td.tt.probe(state.board.hash_key);

    if let Some(TTEntryBucketProbe { entry, .. }) = tt_entry {
        let entry_tt_score = MaybeScore::from_i32(entry.score as i32);
        evaluator_eval = MaybeScore::from_i32(entry.eval as i32);
        tt_move = entry.best_move;
        tt_pv = entry.tt_flag.is_pv();

        // tt-cutoff
        if !NT::IS_PV
            && entry_tt_score.is_some()
            && (
                depth_left.value() <= entry.depth as i32 
                    || entry.quiescence_depth == TTEntry::QUIESCENCE_PROVEN_DEPTH
            )
            && let entry_tt_score = transposition_table::decode_mate_distance(entry_tt_score.unwrap(), td.ply)
            && match entry.tt_flag.maybe_score_kind() {
                Some(ScoreKind::LowerBound) => entry_tt_score >= beta,
                Some(ScoreKind::UpperBound) => entry_tt_score <= alpha,
                Some(ScoreKind::Exact) => true,
                None => false,
            } 
        {
            if entry_tt_score >= beta
                && let Some(pos) = tt_move.ok()
                && threat_kind.is_none()
                && state.board.is_legal_move(pos)
                && !state.board.patterns.field[state.board.player_color][pos.idx_usize()].is_tactical()
            {
                td.push_killer(pos);

                let quiet_plied = Bitfield::unit(pos);
                td.ht.update_quiet(&state.history, quiet_plied, state.board.player_color, pos, depth_left);
            }

            return entry_tt_score;
        }
    } else {
        evaluator_eval = MaybeScore::NONE;
        tt_move = MaybePos::NONE;
        tt_pv = NT::IS_PV;
    }

    let evaluator_eval = evaluator_eval.unwrap_or_else(|| 
        td.evaluator.eval_value(state)
    );

    td.ss[td.ply].evaluator_eval = evaluator_eval.into();

    let static_eval = if td.evaluator.require_stabilize() && td.ply > 0 {
        (-td.ss[td.ply - 1].evaluator_eval.unwrap_or(-evaluator_eval) + evaluator_eval) / 2
    } else {
        evaluator_eval
    };

    td.ss[td.ply].static_eval = static_eval;

    let static_eval_improvement = if td.ply > 1 {
        static_eval - td.ss[td.ply - 2].static_eval
    } else {
        Score::DRAW
    }.value();

    td.ss[td.ply].recovery_state = state.recovery_state();

    td.clear_killer();

    let original_alpha = alpha;
    let mut best_score = Score::NEG_INF;
    let mut best_move = MaybePos::NONE;

    let mut moves_made = 0;
    let mut searched_moves = 0;

    let mut quiet_plied = Bitfield::ZERO_FILLED;
    let mut three_plied = Bitfield::ZERO_FILLED;
    let mut four_plied = Bitfield::ZERO_FILLED;

    let mut move_picker = MovePicker::init_new(tt_move, td.killers[td.ply], threat_kind);
    'position_search: while let Some(MainMoveEntry { pos, score: move_score, history_score, .. }) = move_picker.next(td, state) {
        if !state.board.is_legal_move(pos) {
            continue;
        }

        moves_made += 1;

        let player_pattern = state.board.patterns.field[state.board.player_color][pos.idx_usize()];
        let opponent_pattern = state.board.patterns.field[!state.board.player_color][pos.idx_usize()];

        let on_three = player_pattern.has_open_three();
        let on_four = player_pattern.has_any_four();
        let on_opponent_three = opponent_pattern.has_open_three();

        let is_tactical = on_three | on_four | on_opponent_three;

        // late move pruning
        if !NT::IS_PV
            && !is_tactical
            && threat_kind.is_none()
        {
            // move count pruning
            let lmp_margin = lookup_lmp_mc_table(depth_left, static_eval_improvement > 0);
            if moves_made >= lmp_margin {
                move_picker.skip_lp_quiets();
                continue 'position_search;
            }

            // futility pruning
            let fp_margin = params::FP_BASE + params::FP_MUL * depth_left.value() * depth_left.value();
            if !alpha.is_win()
                 && static_eval + fp_margin <= alpha
            {
                move_picker.skip_lp_quiets();
                continue 'position_search;
            }
        }

        td.tt.prefetch(state.board.hash_key.set(state.board.player_color, pos));

        let artifact = state.play_mut(pos);
        td.push_ply(pos);
        td.evaluator.play(&state.board, artifact, pos.into());

        if threat_kind.is_none() {
            if on_three {
                three_plied.set(pos);
            } else if on_four {
                four_plied.set(pos);
            } else {
                quiet_plied.set(pos);
            }
        }

        let new_full_depth = depth_left - 1;
        let mut reduction = Depth::ZERO;

        // late move reduction
        if depth_left > Depth::from_i32(2)
            && moves_made > 1 + NT::IS_ROOT as usize
            && move_score < move_generator::KILLER_MOVE_SCORE
            && threat_kind.is_none()
        {
            reduction = td.lookup_lmr_table(depth_left, moves_made);

            // cut-node reduction
            reduction += Depth::from_i32(cut_node as i32);

            // reduction pv less
            reduction -= Depth::from_i32(NT::IS_PV as i32);

            // reduction tactical less
            if is_tactical
                && td.ply < 4
            {
                reduction -= Depth::from_i32(1);
            }

            // reduction history score
            if let Some(history_score) = history_score {
                if history_score > 0 {
                    reduction -= Depth::from_i32(1);
                } else if history_score < -4096 {
                    reduction += Depth::from_i32(1);
                }
            }

            reduction = reduction.clamp_value(new_full_depth);
        }

        let new_depth = (new_full_depth - reduction).clamp_value(new_full_depth);

        let nodes_before = td.batch_counter.count_local_in_1k();

        searched_moves += 1;

        let score = if moves_made == 1 { // full-window search
            -pvs::<R, TH, NT::NextType>(td, &mut child_pv, state, new_depth, -beta, -alpha, !NT::IS_PV && !cut_node)
        } else { // zero-window search
            let mut score = -pvs::<R, TH, OffPVNode>(
                td, &mut child_pv, state, new_depth, -alpha - 1, -alpha, true,
            );

            if score > alpha
                && new_depth < new_full_depth
            { // zero-window failed, full-depth null-window search
                score = -pvs::<R, TH, OffPVNode>(
                    td, &mut child_pv, state, new_full_depth, -alpha - 1, -alpha, !cut_node,
                );
            }

            if NT::IS_PV
                && alpha < score && score < beta
            { // exact value required, full-window search
                score = -pvs::<R, TH, NT::NextType>(
                    td, &mut child_pv, state, new_full_depth, -beta, -alpha, false,
                );
            }

            score
        };

        if NT::IS_ROOT {
            td.root_moves_in_1k[pos.idx_usize()] += td.batch_counter.count_local_in_1k() - nodes_before;
        }

        td.pop_ply();
        let artifact = state.undo_mut(td.ss[td.ply].recovery_state);
        td.evaluator.undo(&state.board, artifact, pos.into());

        if td.is_aborted() {
            return Score::DRAW;
        }

        if score <= best_score {
            continue;
        }

        best_score = score;

        if score > alpha { // improve alpha
            best_move = pos.into();
            alpha = score;

            if NT::IS_PV { // update pv-line
                pv.update(pos, &child_pv);
            }

            if alpha >= beta { // beta cutoff
                break 'position_search;
            }
        }
    }

    if moves_made == 0 || searched_moves == 0 {
        best_score = static_eval;
    }

    if NT::IS_ROOT && moves_made == 1 {
        td.singular_root = true;
    }

    let score_kind = if best_score >= beta {
        ScoreKind::LowerBound
    } else if best_score > original_alpha {
        ScoreKind::Exact
    } else {
        ScoreKind::UpperBound
    };

    if alpha > original_alpha {
        let best_move = best_move.unwrap();

        if threat_kind.is_none() {
            if !state.board.patterns.field[state.board.player_color][best_move.idx_usize()].is_tactical() {
                td.push_killer(best_move);
            }

            td.ht.update_tactical(three_plied, four_plied, state.board.player_color, best_move, depth_left);
            td.ht.update_quiet(&state.history, quiet_plied, state.board.player_color, best_move, depth_left)
        }
    }

    td.tt.store(
        state.board.hash_key,
        best_move,
        depth_left,
        0,
        Some(score_kind),
        evaluator_eval.into(),
        transposition_table::encode_mate_distance(best_score, td.ply).into(),
        tt_pv | NT::IS_PV,
    );

    best_score
}

fn lookup_lmp_mc_table(depth: Depth, is_improving: bool) -> usize {
    let clamped_depth = (depth.value() - 1).min(11) as usize;

    LMP_MC_TABLE[is_improving as usize][clamped_depth]
}

const LMP_MC_TABLE: [[usize; 12]; 2] = build_lmp_mc_table();

const fn build_lmp_mc_table() -> [[usize; 12]; 2] {
    let mut lmp_table = [[0; 12]; 2];

    const_for!(depth in 0, 12; {
        let pow_depth = depth as f64 * depth as f64;

        lmp_table[0][depth] = params::LMP_BASE + (pow_depth / params::LMP_DIV_NON_IMPROVING) as usize;
        lmp_table[1][depth] = params::LMP_BASE + (pow_depth / params::LMP_DIV_IMPROVING) as usize;
    });

    lmp_table
}
