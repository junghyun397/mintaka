use crate::eval::evaluator::Evaluator;
use crate::game_state::GameState;
use crate::memo::history_table::{QuietPlied, TacticalPlied};
use crate::memo::transposition_table;
use crate::memo::tt_entry::{ScoreKind, TTFlag};
use crate::movegen::move_generator;
use crate::movegen::move_list::MoveEntry;
use crate::movegen::move_picker::{MovePicker, ThreatKind};
use crate::principal_variation::PrincipalVariation;
use crate::protocol::response::Response;
use crate::search_endgame::endgame_search;
use crate::thread_data::{SearchFrame, ThreadData};
use crate::thread_type::ThreadType;
use crate::utils::time::MonotonicClock;
use crate::value::Depth;
use crate::{params, value};
use rusty_renju::const_for;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::score::{Score, Scores};

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

pub fn iterative_deepening<CLK: MonotonicClock, const R: RuleKind, TH: ThreadType>(
    td: &mut ThreadData<R, TH, impl Evaluator<R>>,
    mut state: GameState<R>,
) -> (Score, MaybePos) {
    let mut score: Score = 0;
    let mut best_move = MaybePos::NONE;
    let mut root_pv = PrincipalVariation::EMPTY;
    let mut selective_depth = 0;

    let mut mate_count = 0;

    let starting_depth = (td.tid % 10 + 1) as Depth;

    'iterative_deepening: for depth in starting_depth ..= td.config.max_depth() {
        let iter_score = if depth < 5 {
            pvs::<CLK, R, TH, RootNode>(td, &mut state, depth, -Score::INF, Score::INF, false)
        } else {
            aspiration::<CLK, R, TH>(td, &mut state, depth, score)
        };

        if td.is_aborted() {
            break 'iterative_deepening;
        }

        score = iter_score;
        best_move = td.best_move;
        root_pv = td.pvs[0];
        selective_depth = td.selective_depth;

        if TH::IS_MAIN {
            td.thread_type.make_response(Response::Status {
                hash: td.thread_type.position_hash(),
                best_move,
                score,
                pv: td.pvs[0],
                total_nodes_in_1k: td.batch_counter.count_global_in_1k(),
                time_elapsed: td.thread_type.time_elapsed(),
                selective_depth: selective_depth as Depth,
            })
        }

        if Score::is_mate(iter_score) {
            mate_count += 1;

            if depth - starting_depth > 10
                && mate_count > 2
            {
                break 'iterative_deepening;
            }
        }

        td.singular_root = false;
    }

    if TH::IS_MAIN {
        td.set_aborted();
    }

    td.selective_depth = selective_depth;
    td.root_pv = root_pv;

    (score, best_move)
}

fn aspiration<CLK: MonotonicClock, const R: RuleKind, TH: ThreadType>(
    td: &mut ThreadData<R, TH, impl Evaluator<R>>,
    state: &mut GameState<R>,
    max_depth: Depth,
    prev_score: Score,
) -> Score {
    let mut depth = max_depth;

    let mut delta = params::ASPIRATION_DELTA_BASE + prev_score.pow(2) / params::ASPIRATION_DELTA_DIV;
    let mut alpha = (prev_score - delta).max(-Score::INF);
    let mut beta = (prev_score + delta).min(Score::INF);

    loop {
        let score = pvs::<CLK, R, TH, RootNode>(td, state, depth, alpha, beta, false);

        if td.is_aborted() {
            return Score::DRAW;
        }

        if score <= alpha { // fail-low
            beta = (alpha + beta) / 2;
            alpha = (score - delta).max(-Score::INF);
            depth = max_depth;
        } else if score >= beta { // fail-high
            beta = (score + delta).min(Score::INF);
            depth = max_depth;
        } else { // exact
            return score;
        }

        delta *= 2;
    }
}

fn pvs<CLK: MonotonicClock, const R: RuleKind, TH: ThreadType, NT: NodeType>(
    td: &mut ThreadData<R, TH, impl Evaluator<R>>,
    state: &mut GameState<R>,
    depth_left: Depth,
    mut alpha: Score,
    mut beta: Score,
    cut_node: bool,
) -> Score {
    if TH::IS_MAIN
        && td.should_check_limit()
        && td.search_limit_exceeded()
    {
        td.set_aborted();
        return Score::ABORT;
    }

    if td.is_aborted() {
        return Score::ABORT;
    }

    if state.board.stones >= td.config.draw_condition as u8 {
        return Score::DRAW;
    }

    td.batch_counter.increment();

    td.pvs[td.ply].clear();

    td.selective_depth = td.selective_depth.max(td.ply);

    {
        let (score, pos) = find_immediate_win(state, td.ply);

        if score != Score::NAN { // immediate win or lose
            if NT::IS_ROOT {
                td.singular_root = true;
                td.best_move = pos;
            }

            return score;
        }

        if let Some(pos) = pos.ok() { // defend immediate win
            if NT::IS_ROOT {
                td.singular_root = true;
            }

            let parent_eval = if td.ply > 0 {
                -td.ss[td.ply - 1].static_eval
            } else {
                0
            };

            td.ss[td.ply] = SearchFrame {
                pos: pos.into(),
                static_eval: parent_eval,
                evaluator_eval: Score::NAN,
                on_pv: NT::IS_PV,
                recovery_state: state.recovery_state(),
                searching: MaybePos::NONE,
            };

            td.push_ply(pos);
            let artifact = state.play_mut(pos);
            td.evaluator.play(&state.board, artifact, pos.into());

            // no depth reduction for forced response
            let score = -pvs::<CLK, R, TH, NT::NextType>(td, state, depth_left, -beta, -alpha, cut_node);

            td.pop_ply();
            let artifact = state.undo_mut(td.ss[td.ply].recovery_state);
            td.evaluator.undo(&state.board, artifact, pos.into());

            if td.is_aborted() {
                return Score::DRAW;
            }

            if NT::IS_PV {
                let sub_pv = td.pvs[td.ply + 1];
                td.pvs[td.ply].load(pos.into(), sub_pv);
            }

            if NT::IS_ROOT {
                td.best_move = pos.into();
            }

            return score;
        }
    }

    if !NT::IS_ROOT {
        alpha = alpha.max(Score::lose_in(td.ply));
        beta = beta.min(Score::win_in(td.ply));
        if alpha >= beta { // mate distance pruning
            return alpha;
        }
    }

    let threat_kind;
    'threat_kind: {
        let fork_four_field = state.board.patterns.effective_fork_four_field(!state.board.player_color);

        if !fork_four_field.is_empty() {
            threat_kind = Some(ThreatKind::ForkFour(fork_four_field));
            break 'threat_kind;
        }

        let fork_three_four_field = state.board.patterns.effective_fork_three_four_field(!state.board.player_color);

        if !fork_three_four_field.is_empty() {
            threat_kind = Some(ThreatKind::ForkThreeFour(fork_three_four_field | state.board.patterns.indexes[!state.board.player_color].open_threes));
            break 'threat_kind;
        }

        threat_kind = None;
    };

    let static_eval: Score;
    let tt_move: MaybePos;
    let tt_pv: bool;
    let tt_endgame_depth: u8;

    let tt_entry = td.tt.probe(state.board.hash_key);

    // endgame-hit
    if let Some(entry) = tt_entry && entry.tt_flag.is_endgame_proven() {
        if NT::IS_ROOT {
            td.best_move = entry.best_move;
            td.singular_root = true;
        }

        if NT::IS_PV {
            td.pvs[td.ply].load(entry.best_move, PrincipalVariation::EMPTY);
        }

        return transposition_table::decode_mate_distance(entry.score as Score, td.ply);
    }

    match tt_entry {
        Some(entry) if entry.tt_flag.maybe_score_kind().is_some() => { // full-tt
            let tt_score = transposition_table::decode_mate_distance(entry.score as Score, td.ply);

            tt_move = entry.best_move;
            tt_pv = entry.tt_flag.is_pv();
            tt_endgame_depth = entry.tt_flag.endgame_depth();

            // tt-cutoff
            if !NT::IS_PV
                && depth_left <= entry.depth as Depth
                && match entry.tt_flag.score_kind() {
                    ScoreKind::LowerBound => tt_score >= beta,
                    ScoreKind::UpperBound => tt_score <= alpha,
                    ScoreKind::Exact => true,
                }
            {
                if tt_score >= beta
                    && let Some(pos) = tt_move.ok()
                    && threat_kind.is_none()
                    && state.board.is_legal_move(pos)
                    && !state.board.patterns.field[state.board.player_color][pos.idx_usize()].is_tactical()
                {
                    td.push_killer(pos);

                    let mut quiet_plied = QuietPlied::EMPTY;
                    quiet_plied.push(pos);
                    td.ht.update_quiet(&state.history, quiet_plied, state.board.player_color, pos, depth_left);
                }

                return tt_score;
            }

            static_eval = entry.eval as Score;
        }
        Some(entry) => { // endgame-tt
            tt_move = MaybePos::NONE;
            tt_pv = false;
            tt_endgame_depth = entry.tt_flag.endgame_depth();

            static_eval = entry.eval as Score;

            td.tt.store(
                state.board.hash_key,
                MaybePos::NONE,
                None,
                tt_endgame_depth,
                0,
                static_eval,
                0,
                false,
            );
        }
        None => {
            tt_move = MaybePos::NONE;
            tt_pv = false;
            tt_endgame_depth = 0;

            let evaluator_eval = td.evaluator.eval_value(state);
            td.ss[td.ply].evaluator_eval = evaluator_eval;

            if td.evaluator.require_stabilize() && td.ply > 0 {
                static_eval = (-td.ss[td.ply - 1].evaluator_eval + evaluator_eval) / 2;
            } else {
                static_eval = evaluator_eval;
            }

            td.tt.store(
                state.board.hash_key,
                MaybePos::NONE,
                None,
                0,
                0,
                static_eval,
                0,
                false,
            );
        }
    }

    td.ss[td.ply].static_eval = static_eval;

    let static_eval_improvement = if td.ply > 1 {
        static_eval - td.ss[td.ply - 2].static_eval
    } else {
        0
    };

    if depth_left <= 0 || td.ply >= value::MAX_PLY {
        let vcf_depth = td.config.max_vcf_depth.unwrap_or(30);

        if static_eval >= beta
            || Score::is_winning(alpha)
            || tt_endgame_depth >= vcf_depth.min(TTFlag::MAX_TT_ENDGAME_DEPTH as Depth) as u8
        {
            return static_eval;
        }

        return endgame_search::<R, false>(
            td, vcf_depth, state, alpha, beta, static_eval
        );
    }

    td.ss[td.ply].recovery_state = state.recovery_state();

    td.clear_killer();

    let original_alpha = alpha;
    let mut best_score = -Score::INF;
    let mut best_move = MaybePos::NONE;

    let mut move_picker = MovePicker::init_new(tt_move, td.killers[td.ply], threat_kind);
    let mut moves_made = 0;

    let mut quiet_plied = QuietPlied::EMPTY;
    let mut three_plied = TacticalPlied::EMPTY;
    let mut four_plied = TacticalPlied::EMPTY;

    'position_search: while let Some(MoveEntry { pos, move_score, history_score, .. }) = move_picker.next(td, state) {
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
            let fp_margin = params::FP_BASE + params::FP_MUL * depth_left * depth_left;
            if !Score::is_winning(alpha)
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
                three_plied.push(pos);
            } else if on_four {
                four_plied.push(pos);
            } else {
                quiet_plied.push(pos);
            }
        }

        let new_full_depth = depth_left - 1;
        let mut reduction = 0;

        // late move reduction
        if depth_left > 2
            && moves_made > 1 + NT::IS_ROOT as usize
            && move_score < move_generator::KILLER_MOVE_SCORE
            && threat_kind.is_none()
        {
            reduction = td.lookup_lmr_table(depth_left, moves_made);

            // cut-node reduction
            reduction += Depth::from(cut_node);

            // reduction pv less
            reduction -= Depth::from(NT::IS_PV);

            // reduction tactical less
            if is_tactical
                && td.ply < 4
            {
                reduction -= 1;
            }

            // reduction history score
            if let Some(history_score) = history_score {
                if history_score > 0 {
                    reduction -= 1;
                } else if history_score < -4096 {
                    reduction += 1;
                }
            }

            reduction = reduction.clamp(0, new_full_depth);
        }

        let new_depth = (new_full_depth - reduction).clamp(0, new_full_depth);

        let score = if moves_made == 1 { // full-window search
            -pvs::<CLK, R, TH, NT::NextType>(td, state, new_depth, -beta, -alpha, !NT::IS_PV && !cut_node)
        } else { // zero-window search
            let mut score = -pvs::<CLK, R, TH, OffPVNode>(
                td, state, new_depth, -alpha - 1, -alpha, true
            );

            if score > alpha
                && new_depth < new_full_depth
            { // zero-window failed, full-depth null-window search
                score = -pvs::<CLK, R, TH, OffPVNode>(
                    td, state, new_full_depth, -alpha - 1, -alpha, !cut_node
                );
            }

            if NT::IS_PV
                && alpha < score && score < beta
            { // exact value required, full-window search
                score = -pvs::<CLK, R, TH, NT::NextType>(
                    td, state, new_full_depth, -beta, -alpha, false,
                );
            }

            score
        };

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
                let sub_pv = td.pvs[td.ply + 1];
                td.pvs[td.ply].load(pos.into(), sub_pv);
            }

            if alpha >= beta { // beta cutoff
                break 'position_search;
            }
        }
    }

    if moves_made == 0 {
        best_score = static_eval;
    }

    if NT::IS_ROOT {
        td.best_move = best_move;

        if moves_made == 1 {
            td.singular_root = true;
        }
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
        Some(score_kind),
        tt_endgame_depth,
        depth_left,
        static_eval,
        transposition_table::encode_mate_distance(best_score, td.ply),
        tt_pv | NT::IS_PV,
    );

    best_score
}

fn find_immediate_win<const R: RuleKind>(state: &GameState<R>, ply: usize) -> (Score, MaybePos) {
    if let Some(pos) = state.board.patterns.unchecked_five_pos[state.board.player_color].ok()
    { // five
        return (Score::win_in(ply + 1), pos.into())
    }

    if let Some(pos) = state.board.patterns.unchecked_five_pos[!state.board.player_color].ok() {
        if state.board.player_color == Color::Black
            && state.board.patterns.is_forbidden(pos)
        { // trap
            return (Score::lose_in(ply + 2), MaybePos::NONE)
        }

        if 1 < state.board.patterns.field[!state.board.player_color].iter()
            .filter(|pattern| pattern.has_five())
            .count()
        { // opponent-five
            return (Score::lose_in(ply + 2), pos.into())
        }

        return (Score::NAN, pos.into())
    }

    if let Some(pos) = state.board.patterns.effective_fork_four_field(state.board.player_color)
        .first_pos()
    { // open-four
        return (Score::win_in(ply + 3), pos.into());
    }

    (Score::NAN, MaybePos::NONE)
}

fn lookup_lmp_mc_table(depth: Depth, is_improving: bool) -> usize {
    let clamped_depth = (depth - 1).min(11) as usize;

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
