use crate::endgame::vcf_search::vcf_search;
use crate::eval::evaluator::Evaluator;
use crate::eval::heuristic_evaluator::HeuristicEvaluator;
use crate::game_state::GameState;
use crate::memo::tt_entry::{EndgameFlag, ScoreKind};
use crate::movegen::move_picker::MovePicker;
use crate::parameters::{ASPIRATION_INITIAL_DELTA, MAX_PLY};
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::value::{Depth, Score, Scores};

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
    td: &mut ThreadData<TH>,
    state: &mut GameState,
) -> Score {
    let mut score: Score = 0;

    'iterative_deepening: for depth in 1 ..= MAX_PLY {
        td.depth = depth;
        score = if depth < 7 {
            pvs::<R, RootNode, TH>(td, state, depth, -Score::INF, Score::INF)
        } else {
            aspiration::<R, TH>(td, state, depth, score)
        };

        if td.is_aborted() {
            break 'iterative_deepening;
        }
    }

    score
}

pub fn aspiration<const R: RuleKind, TH: ThreadType>(
    td: &mut ThreadData<TH>,
    state: &mut GameState,
    max_depth: usize,
    prev_score: Score,
) -> Score {
    let mut delta = ASPIRATION_INITIAL_DELTA + prev_score / 1024;
    let mut alpha = (prev_score - delta).max(-Score::INF);
    let mut beta = (prev_score + delta).min(Score::INF);
    let mut depth = max_depth;
    let min_depth = (depth / 2).max(1);

    loop {
        let score = pvs::<R, RootNode, TH>(td, state, depth, alpha, beta);

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

pub fn pvs<const R: RuleKind, NT: NodeType, TH: ThreadType>(
    td: &mut ThreadData<TH>,
    state: &mut GameState,
    depth_left: usize,
    mut alpha: Score,
    mut beta: Score,
) -> Score {
    if td.config.draw_condition.is_some_and(|depth|
        state.history.len() + 1 >= depth
    )
        || state.board.stones == pos::U8_BOARD_SIZE
    {
        return Score::DRAW;
    }

    if let &Some(pos) = state.board.patterns.unchecked_five_pos
        .access(state.board.player_color)
    { // immediate win
        return Score::win_in(td.ply + 1)
    }

    if let &Some(pos) = state.board.patterns.unchecked_five_pos
        .access(!state.board.player_color)
    { // defend immediate win
        if state.board.player_color == Color::Black
            && state.board.patterns.forbidden_field.is_hot(pos)
        { // trapped
            return Score::lose_in(td.ply + 2)
        }

        td.push_ply_mut();
        td.batch_counter.increment_single_mut();
        state.set_mut(pos);

        return -pvs::<R, NT::NextType, TH>(td, state, depth_left, -beta, -alpha);
    }

    // clear pv-line
    td.pvs[td.ply].clear();

    if !NT::IS_ROOT {
        if td.ply >= MAX_PLY {
            return HeuristicEvaluator.eval_value(&state.board);
        }

        alpha = alpha.max(Score::lose_in(td.ply));
        beta = beta.min(Score::win_in(td.ply));
        if alpha >= beta { // mate distance pruning
            return alpha;
        }
    }

    if TH::IS_MAIN
        && td.should_check_limit()
        && td.search_limit_exceeded()
    {
        td.set_aborted_mut();
        return 0;
    } else if td.is_aborted() {
        return 0;
    }

    let mut static_eval: Score;
    let mut tt_move: MaybePos;
    let mut tt_pv: bool;
    let mut tt_endgame_flag: EndgameFlag;

    if let Some(entry) = td.tt.probe(state.board.hash_key) {
        tt_endgame_flag = entry.tt_flag.endgame_flag();
        if tt_endgame_flag == EndgameFlag::Win || tt_endgame_flag == EndgameFlag::Lose
        { // endgame tt-hit
            return entry.score;
        }

        static_eval = entry.eval;
        tt_move = entry.best_move;
        tt_pv = entry.tt_flag.is_pv();

        if match entry.tt_flag.score_kind() {
            ScoreKind::LowerBound => entry.score >= beta,
            ScoreKind::UpperBound => entry.score <= alpha,
            ScoreKind::Exact => true,
        } {
            return entry.score;
        }

        static_eval = entry.score;
    } else {
        tt_endgame_flag = EndgameFlag::Unknown;
        static_eval = HeuristicEvaluator.eval_value(&state.board);
        tt_move = MaybePos::NONE;
        tt_pv = false;
    }

    if depth_left == 0 {
        return vcf_search(td, td.config.max_vcf_depth, state, alpha, beta)
            .unwrap_or(static_eval);
    }

    let original_alpha = alpha;

    let mut best_score = -Score::INF;
    let mut best_move = MaybePos::NONE;

    td.killers[td.ply] = [MaybePos::NONE; 2]; // todo: DEBUG
    let mut move_picker = MovePicker::new(tt_move, td.killers[td.ply]);

    let mut moves = 0;

    td.push_ply_mut();
    td.ss[td.ply].static_eval = static_eval;
    td.ss[td.ply].on_pv = NT::IS_PV || tt_pv;

    'position_search: while let Some((pos, _)) = move_picker.next(state) {
        if !state.board.is_legal_move(pos) {
            continue;
        }

        td.tt.prefetch(state.board.hash_key.set(state.board.player_color, pos));

        let movegen_window = state.movegen_window;
        td.batch_counter.increment_single_mut();
        state.set_mut(pos);

        moves += 1;

        let score = if moves == 1 { // full-window search
            -pvs::<R, NT::NextType, TH>(td, state, depth_left - 1, -beta, -alpha)
        } else { // zero-window search
            let mut score = -pvs::<R, OffPVNode, TH>(td, state, depth_left - 1, -alpha - 1, -alpha);

            if score > alpha { // zero-window failed, full-window search
                score = -pvs::<R, PVNode, TH>(td, state, depth_left - 1, -beta, -alpha);
            }

            score
        };

        state.unset_mut(movegen_window);

        if score <= best_score {
            continue;
        }

        best_score = score;

        if score > alpha { // improve alpha
            best_move = pos.into();
            alpha = score;

            if NT::IS_PV { // update pv-line
                let sub_pv = td.pvs[td.ply].clone();
                td.pvs[td.ply - 1].load(pos.into(), &sub_pv);
            }
        }

        if alpha >= beta { // beta cutoff
            td.ss[td.ply].cutoffs += 1;
            td.insert_killer_move_mut(pos);
            break 'position_search;
        }
    }

    td.ss[td.ply].best_move = best_move;

    if NT::IS_ROOT {
        td.best_move = best_move;
    }

    td.pop_ply_mut();

    let score_kind = if best_score >= beta {
        ScoreKind::LowerBound
    } else if best_score > original_alpha {
        ScoreKind::Exact
    } else {
        ScoreKind::UpperBound
    };

    td.tt.store_mut(
        state.board.hash_key,
        best_move,
        score_kind,
        tt_endgame_flag,
        td.ply as Depth,
        static_eval,
        best_score,
        NT::IS_PV
    );

    best_score
}
