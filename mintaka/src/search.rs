use crate::endgame::vcf_search::vcf_search;
use crate::eval::evaluator::Evaluator;
use crate::game_state::GameState;
use crate::memo::tt_entry::ScoreKind;
use crate::movegen::move_list::MoveEntry;
use crate::movegen::move_picker::MovePicker;
use crate::protocol::response::Response;
use crate::search_frame::SearchFrame;
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use crate::value;
use crate::value::Depth;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::value::{Score, Scores};

pub trait NodeType {

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

pub fn iterative_deepening<const R: RuleKind, TH: ThreadType>(
    td: &mut ThreadData<TH, impl Evaluator>,
    mut state: GameState,
) -> (Score, MaybePos) {
    let mut score: Score = 0;
    let mut best_move = MaybePos::NONE;

    'iterative_deepening: for depth in 1 ..= td.config.max_depth {
        td.depth = depth;

        let iter_score = if depth < 7 {
            pvs::<R, TH, RootNode>(td, &mut state, depth, -Score::INF, Score::INF)
        } else {
            aspiration::<R, TH>(td, &mut state, depth, score)
        };

        if td.is_aborted() {
            break 'iterative_deepening;
        }

        score = iter_score;
        best_move = td.best_move;
        td.depth_reached = depth;

        if TH::IS_MAIN {
            td.thread_type.make_response(Response::Status {
                best_move,
                score,
                pv: td.pvs[0],
                total_nodes_in_1k: td.batch_counter.count_global_in_1k(),
                depth
            })
        }
    }

    (score, best_move)
}

pub fn aspiration<const R: RuleKind, TH: ThreadType>(
    td: &mut ThreadData<TH, impl Evaluator>,
    state: &mut GameState,
    max_depth: Depth,
    prev_score: Score,
) -> Score {
    let mut delta = value::ASPIRATION_INITIAL_DELTA + prev_score / 1024;
    let mut alpha = (prev_score - delta).max(-Score::INF);
    let mut beta = (prev_score + delta).min(Score::INF);
    let mut depth = max_depth;
    let min_depth = (depth / 2).max(1);

    loop {
        let score = pvs::<R, TH, RootNode>(td, state, depth, alpha, beta);

        if td.is_aborted() {
            return score;
        }

        if score <= alpha { // fail-low
            beta = (alpha + beta) / 2;
            alpha = (alpha - delta).max(-Score::INF);
            depth = max_depth;
        } else if score >= beta { // fail-high
            beta = (beta + delta).min(Score::INF);
            depth = (depth - 1).max(min_depth);
        } else { // expected
            return score;
        }

        delta += delta / 2;
    }
}

pub fn pvs<const R: RuleKind, TH: ThreadType, NT: NodeType>(
    td: &mut ThreadData<TH, impl Evaluator>,
    state: &mut GameState,
    depth_left: Depth,
    mut alpha: Score,
    mut beta: Score,
) -> Score {
    if TH::IS_MAIN
        && td.should_check_limit()
        && td.search_limit_exceeded()
    {
        td.set_aborted_mut();
        return Score::DRAW;
    }

    if td.is_aborted() || state.board.stones as usize >= td.config.draw_condition {
        return Score::DRAW;
    }

    td.pvs[td.ply].clear();

    if let &Some(pos) = state.board.patterns.unchecked_five_pos
        .access(state.board.player_color)
    { // immediate win
        if NT::IS_ROOT {
            td.best_move = pos.into();
        }

        return Score::win_in(td.ply + 1)
    }

    if let &Some(pos) = state.board.patterns.unchecked_five_pos
        .access(!state.board.player_color)
    { // defend immediate win
        if state.board.player_color == Color::Black
            && state.board.patterns.forbidden_field.is_hot(pos)
        { // trapped
            if NT::IS_ROOT {
                td.best_move = MaybePos::NONE;
            }

            return Score::lose_in(td.ply + 2)
        }

        td.ss[td.ply] = SearchFrame {
            hash_key: state.board.hash_key,
            static_eval: Score::lose_in(td.ply + 1),
            on_pv: NT::IS_PV,
            movegen_window: state.movegen_window,
            last_pos: pos.into(),
            cutoffs: 0,
        };

        td.push_ply_mut(pos);
        state.set_mut(pos);

        let score = -pvs::<R, TH, NT::NextType>(td, state, depth_left, -beta, -alpha);

        td.pop_ply_mut();
        state.unset_mut(td.ss[td.ply].movegen_window);

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

    let mut static_eval: Score;
    let mut tt_move: MaybePos;
    let mut tt_pv: bool;
    let mut tt_endgame_visited: bool;

    if let Some(entry) = td.tt.probe(state.board.hash_key) { // tt-lookup
        let tt_score = entry.score as Score;

        tt_move = entry.best_move;
        tt_pv = entry.tt_flag.is_pv();
        tt_endgame_visited = entry.tt_flag.endgame_visited();

        if !NT::IS_PV // tt-cutoff
            && match entry.tt_flag.score_kind() {
                ScoreKind::LowerBound => tt_score >= beta,
                ScoreKind::UpperBound => tt_score <= alpha,
                ScoreKind::Exact => true,
            }
        {
            return tt_score;
        }

        static_eval = entry.eval as Score;
    } else {
        tt_move = MaybePos::NONE;
        tt_pv = false;
        tt_endgame_visited = false;

        static_eval = td.evaluator.eval_value(state);
    }

    if depth_left <= 0 || td.ply >= value::MAX_PLY {
        return vcf_search::<R>(td, td.config.max_vcf_depth, state, alpha, beta, static_eval);
    }

    let mut best_score = -Score::INF;
    let mut best_move = MaybePos::NONE;

    let mut score_kind = ScoreKind::UpperBound;

    td.ss[td.ply] = SearchFrame {
        movegen_window: state.movegen_window,
        hash_key: state.board.hash_key,
        static_eval,
        on_pv: NT::IS_PV || tt_pv,
        last_pos: MaybePos::NONE,
        cutoffs: 0,
    };

    let mut move_picker = MovePicker::new(tt_move, td.killers[td.ply]);
    let mut moves_made = 0;

    'position_search: while let Some(MoveEntry { pos, .. }) = move_picker.next(td, state) {
        if !state.board.is_legal_move(pos) {
            continue;
        }

        td.tt.prefetch(state.board.hash_key.set(state.board.player_color, pos));

        td.ss[td.ply].last_pos = pos.into();

        td.push_ply_mut(pos);
        state.set_mut(pos);

        moves_made += 1;

        let score = if moves_made == 1 { // full-window search
            -pvs::<R, TH, NT::NextType>(td, state, depth_left - 1, -beta, -alpha)
        } else { // zero-window search
            let mut score = -pvs::<R, TH, OffPVNode>(td, state, depth_left - 1, -alpha - 1, -alpha);

            if score > alpha { // zero-window failed, full-window search
                score = -pvs::<R, TH, NT::NextType>(td, state, depth_left - 1, -beta, -alpha);
            }

            score
        };

        td.pop_ply_mut();
        state.unset_mut(td.ss[td.ply].movegen_window);

        if td.is_aborted() {
            return Score::DRAW;
        }

        if NT::IS_ROOT {
            td.push_root_move(pos, score);
        }

        if score <= best_score {
            continue;
        }

        best_score = score;

        if score > alpha { // improve alpha
            score_kind = ScoreKind::Exact;

            best_move = pos.into();
            alpha = score;

            if NT::IS_PV { // update pv-line
                let sub_pv = td.pvs[td.ply + 1];
                td.pvs[td.ply].load(pos.into(), sub_pv);
            }

            if alpha >= beta { // beta cutoff
                score_kind = ScoreKind::LowerBound;

                td.ss[td.ply].cutoffs += 1;
                td.push_killer_move_mut(pos);
                break 'position_search;
            }
        }
    }

    if moves_made == 0 {
        best_score = static_eval;
    }

    if NT::IS_ROOT {
        td.best_move = best_move;
    }

    td.tt.store(
        state.board.hash_key,
        best_move,
        score_kind,
        tt_endgame_visited,
        depth_left,
        static_eval,
        best_score,
        NT::IS_PV
    );

    best_score
}
