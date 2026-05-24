use crate::eval::evaluator::Evaluator;
use crate::memo::history_table::{QuietPlied, TacticalPlied};
use crate::memo::tt_entry::ScoreKind;
use crate::movegen::move_list::MoveEntry;
use crate::movegen::move_picker;
use crate::movegen::move_picker::MovePicker;
use crate::principal_variation::PrincipalVariation;
use crate::protocol::response::Response;
use crate::search_endgame::endgame_search;
use crate::game_state::GameState;
use crate::thread_data::{SearchFrame, ThreadData};
use crate::thread_type::ThreadType;
use crate::utils::time::MonotonicClock;
use crate::value::Depth;
use crate::{params, value};
use rusty_renju::const_for;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::score::{Score, Scores};
use crate::memo::transposition_table::{decode_mate_distance, encode_mate_distance};
use crate::search_snap::find_immediate_win;

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
    td: &mut ThreadData<TH, impl Evaluator>,
    mut state: GameState,
) -> (Score, MaybePos) {
    let mut score: Score = 0;
    let mut best_move = MaybePos::NONE;
    let mut root_pv = PrincipalVariation::EMPTY;
    let mut selective_depth = 0;

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

        if td.singular_root && Score::is_winning(iter_score) {
            break 'iterative_deepening;
        }
    }

    if TH::IS_MAIN {
        td.set_aborted();
    }

    td.selective_depth = selective_depth;
    td.root_pv = root_pv;

    (score, best_move)
}

fn aspiration<CLK: MonotonicClock, const R: RuleKind, TH: ThreadType>(
    td: &mut ThreadData<TH, impl Evaluator>,
    state: &mut GameState,
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
    td: &mut ThreadData<TH, impl Evaluator>,
    state: &mut GameState,
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

        if score != Score::NAN {
            if NT::IS_ROOT {
                td.singular_root = true;
                td.best_move = pos;
            }

            return score;
        }
    }

    if let Some(pos) = state.board.patterns.unchecked_five_pos[!state.board.player_color].ok()
        && td.ply < value::MAX_PLY
    { // defend immediate win
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
            on_pv: NT::IS_PV,
            recovery_state: state.recovery_state(),
            searching: MaybePos::NONE,
        };

        td.push_ply(pos);
        state.play_mut(pos);

        // no depth reduction for forced defense
        let score = -pvs::<CLK, R, TH, NT::NextType>(td, state, depth_left, -beta, -alpha, cut_node);

        td.pop_ply();
        state.undo_mut(td.ss[td.ply].recovery_state);

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

    if !NT::IS_ROOT {
        alpha = alpha.max(Score::lose_in(td.ply));
        beta = beta.min(Score::win_in(td.ply));
        if alpha >= beta { // mate distance pruning
            return alpha;
        }
    }

    let forced_defense = state.board.patterns.effective_open_fours(!state.board.player_color) != 0;

    let static_eval: Score;
    let tt_move: MaybePos;
    let tt_pv: bool;
    let tt_endgame_depth: Depth;

    match td.tt.probe(state.board.hash_key) {
        Some(entry) if entry.tt_flag.maybe_score_kind().is_some() => { // full-tt
            let tt_score = decode_mate_distance(entry.score as Score, td.ply);

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
                return tt_score;
            }

            static_eval = entry.eval as Score;
        }
        Some(entry) => {
            tt_move = MaybePos::NONE;
            tt_pv = false;
            tt_endgame_depth = entry.tt_flag.endgame_depth();

            static_eval = td.evaluator.eval_value(state);

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

            static_eval = td.evaluator.eval_value(state);

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
        return endgame_search::<R, false>(
            td, td.config.max_vcf_depth.unwrap_or(pos::BOARD_SIZE as Depth), state, alpha, beta, static_eval
        );
    }

    td.ss[td.ply].recovery_state = state.recovery_state();

    td.clear_killer();

    let original_alpha = alpha;
    let mut best_score = -Score::INF;
    let mut best_move = MaybePos::NONE;

    let mut move_picker = MovePicker::init_new(tt_move, td.killers[td.ply], forced_defense);
    let mut moves_made = 0;

    let mut quiet_plied = QuietPlied::EMPTY;
    let mut three_plied = TacticalPlied::EMPTY;
    let mut four_plied = TacticalPlied::EMPTY;

    'position_search: while let Some(MoveEntry { pos, move_score }) = move_picker.next(td, state) {
        if !state.board.is_legal_move(pos) {
            continue;
        }

        moves_made += 1;

        let player_pattern = state.board.patterns.field[state.board.player_color][pos.idx_usize()];
        let on_three = player_pattern.has_three();
        let on_four = player_pattern.has_any_four();
        let is_tactical = on_three | on_four;

        if !NT::IS_PV
            && !is_tactical
            && !forced_defense
            && move_score < move_picker::KILLER_MOVE_SCORE
        {
            // move count pruning
            let lmp_margin = lookup_lmp_mc_table(depth_left, static_eval_improvement > 0);
            if moves_made >= lmp_margin {
                break 'position_search;
            }

            // futility pruning
            let fp_margin = params::FP_BASE + params::FP_MUL * depth_left * depth_left;
            if !Score::is_winning(alpha)
                 && static_eval + fp_margin <= alpha
            {
                break 'position_search;
            }
        }

        td.tt.prefetch(state.board.hash_key.set(state.board.player_color, pos));

        td.push_ply(pos);
        state.play_mut(pos);

        if on_three {
            three_plied.push(pos);
        }

        if on_four {
            four_plied.push(pos);
        }

        if !on_three && !on_four {
            quiet_plied.push(pos);
        }

        let reduced_depth_left = {
            let mut reduction = 1;

            // late move reduction
            if depth_left > 2
                && move_score < move_picker::KILLER_MOVE_SCORE
                && (!NT::IS_ROOT || moves_made > 2)
            {
                // base reduction
                reduction += td.lookup_lmr_table(depth_left, moves_made);

                // cut-node reduction
                reduction += Depth::from(cut_node);

                // reduction pv less
                reduction -= Depth::from(NT::IS_PV);

                if NT::IS_ROOT {
                    reduction -= 1;
                }

                // reduction tactical less
                if is_tactical
                    && td.ply < 4
                {
                    reduction -= 1;
                }

                reduction = reduction.clamp(1, depth_left - 1);
            }

            depth_left - reduction
        };

        let score = if moves_made == 1 { // full-window search
            -pvs::<CLK, R, TH, NT::NextType>(td, state, reduced_depth_left, -beta, -alpha, false)
        } else { // zero-window search
            let mut score = -pvs::<CLK, R, TH, OffPVNode>(
                td, state, reduced_depth_left, -alpha - 1, -alpha, true
            );

            if score > alpha
                && reduced_depth_left < depth_left - 1
            { // zero-window failed, full-depth null-window search
                score = -pvs::<CLK, R, TH, OffPVNode>(
                    td, state, depth_left - 1, -alpha - 1, -alpha, !cut_node
                );
            }

            if NT::IS_PV
                && alpha < score && score < beta
            { // exact value required, full-window search
                score = -pvs::<CLK, R, TH, NT::NextType>(
                    td, state, depth_left - 1, -beta, -alpha, false,
                );
            }

            score
        };

        td.pop_ply();
        state.undo_mut(td.ss[td.ply].recovery_state);

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

        td.push_killer(best_move);

        if !forced_defense {
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
        encode_mate_distance(best_score, td.ply),
        tt_pv | NT::IS_PV,
    );

    best_score
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
