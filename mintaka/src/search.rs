use crate::eval::evaluator::Evaluator;
use crate::memo::history_table::{QuietPlied, TacticalPlied};
use crate::memo::transposition_table::TTHit;
use crate::memo::tt_entry::ScoreKind;
use crate::movegen::move_list::MoveEntry;
use crate::movegen::move_picker;
use crate::movegen::move_picker::MovePicker;
use crate::principal_variation::PrincipalVariation;
use crate::protocol::response::Response;
use crate::search_endgame::endgame_search;
use crate::state::GameState;
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
    td: &mut ThreadData<TH, impl Evaluator>,
    mut state: GameState,
) -> (Score, MaybePos) {
    let mut score: Score = 0;
    let mut best_move = MaybePos::NONE;
    let mut root_pv = PrincipalVariation::EMPTY;
    let mut selective_depth = 0;

    let starting_depth = (td.tid % 10 + 1) as Depth;

    'iterative_deepening: for depth in starting_depth ..= td.config.max_depth {
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
    let min_depth = (max_depth / 2).max(1);
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
            depth = (depth - 1).max(min_depth);
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
        return Score::DRAW;
    }

    if td.is_aborted() || state.board.stones >= td.config.draw_condition as u8 {
        return Score::DRAW;
    }

    td.pvs[td.ply].clear();

    if td.selective_depth < td.ply {
        td.selective_depth = td.ply;
    }

    if let Some(pos) = state.board.patterns.unchecked_five_pos[state.board.player_color]
    { // immediate win
        if NT::IS_ROOT {
            td.singular_root = true;
            td.best_move = pos.into();
        }

        return Score::win_in(td.ply + 1)
    }

    if let Some(pos) = state.board.patterns.unchecked_five_pos[!state.board.player_color]
        && td.ply < value::MAX_PLY
    { // defend immediate win
        if NT::IS_ROOT {
            td.singular_root = true;
        }

        if state.board.player_color == Color::Black
            && state.board.patterns.forbidden_field.is_hot(pos)
        { // trapped
            if NT::IS_ROOT {
                td.best_move = MaybePos::NONE;
            }

            return Score::lose_in(td.ply + 2)
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
            cutoffs: 0,
        };

        td.push_ply(pos);
        state.set_mut(pos);

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

    let static_eval: Score;
    let tt_move: MaybePos;
    let tt_pv: bool;
    let tt_vcf_depth: Depth;

    match td.tt.probe(state.board.hash_key) {
        TTHit::Eval(tt_eval) => {
            tt_move = MaybePos::NONE;
            tt_pv = false;
            tt_vcf_depth = 0;

            static_eval = tt_eval;
        }
        TTHit::Entry(entry) => {
            let tt_score = entry.score as Score;

            tt_move = entry.best_move;
            tt_pv = entry.tt_flag.is_pv();
            tt_vcf_depth = entry.tt_flag.endgame_depth();

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

            static_eval = entry.eval();
        },
        TTHit::None => {
            tt_move = MaybePos::NONE;
            tt_pv = false;
            tt_vcf_depth = 0;

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
        return endgame_search::<R, false>(td, td.config.max_vcf_depth, state, static_eval, alpha, beta);
    }

    td.ss[td.ply].recovery_state = state.recovery_state();

    td.clear_killer();

    let forced_defense = state.board.is_forced_defense();

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

        let player_pattern = state.board.patterns.field[state.board.player_color][pos.idx_usize()];
        let on_three = player_pattern.has_three();
        let on_four = player_pattern.has_any_four();
        let is_tactical = on_three | on_four;

        if !NT::IS_PV
            && !Score::is_losing(best_score)
            && move_score < move_picker::KILLER_MOVE_SCORE
        {
            // move count pruning
            let lmp_margin = lookup_lmp_mc_table(depth_left, static_eval_improvement > 0);
            if moves_made >= lmp_margin {
                break;
            }

            // futility pruning
            let fp_margin = params::FP_BASE + params::FP_MUL * depth_left * depth_left;
            if !Score::is_winning(alpha)
                 && static_eval + fp_margin <= alpha
            {
                break;
            }
        }

        td.tt.prefetch(state.board.hash_key.set(state.board.player_color, pos));

        td.push_ply(pos);
        state.set_mut(pos);

        moves_made += 1;

        if on_three {
            three_plied.push(pos);
        } else if on_four {
            four_plied.push(pos);
        } else {
            quiet_plied.push(pos);
        }

        let reduced_depth_left = {
            let mut reduction = 1;

            // late move reduction
            if !NT::IS_ROOT
                && move_score < move_picker::KILLER_MOVE_SCORE
                && depth_left > 1
            {
                // base reduction
                reduction += td.lookup_lmr_table(depth_left, moves_made);

                // cut node reduction
                reduction += Depth::from(cut_node);

                // reduction pv less
                reduction -= Depth::from(NT::IS_PV);

                // reduction tactical less
                if is_tactical
                    && td.ply < 4
                {
                    reduction -= 1;
                }

                // reduction sparse fail-highs
                reduction -= (td.ss[td.ply].cutoffs < 4) as Depth;

                reduction = reduction.max(1);
            }

            depth_left - reduction
        };

        let score = if moves_made == 1 { // full-window search
            -pvs::<CLK, R, TH, NT::NextType>(td, state, reduced_depth_left, -beta, -alpha, false)
        } else { // zero-window search
            let mut score = -pvs::<CLK, R, TH, OffPVNode>(td, state, reduced_depth_left, -alpha - 1, -alpha, true);

            if score > alpha { // zero-window failed, full-window search
                score = -pvs::<CLK, R, TH, NT::NextType>(td, state, reduced_depth_left, -beta, -alpha, false);
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
                td.ss[td.ply].cutoffs += 1;

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
        let best_move_pattern = state.board.patterns.field[state.board.player_color][best_move.idx_usize()];

        td.push_killer(best_move);

        if best_move_pattern.is_tactical() {
            td.ht.update_tactical(three_plied, four_plied, state.board.player_color, best_move, depth_left);
        } else {
            td.ht.update_quiet(&state.history, quiet_plied, state.board.player_color, best_move, depth_left)
        }
    }

    td.tt.store(
        state.board.hash_key,
        best_move,
        Some(score_kind),
        tt_vcf_depth,
        depth_left,
        static_eval,
        best_score,
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
