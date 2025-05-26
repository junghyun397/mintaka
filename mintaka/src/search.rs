use crate::endgame::vcf_search::vcf_search;
use crate::eval::evaluator::Evaluator;
use crate::eval::heuristic_evaluator::HeuristicEvaluator;
use crate::game_state::GameState;
use crate::memo::tt_entry::{EndgameFlag, ScoreKind};
use crate::movegen::move_picker::MovePicker;
use crate::parameters::{ASPIRATION_WINDOW_BASE_DELTA, MAX_PLY};
use crate::principal_variation::PrincipalVariation;
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
    let mut score: Score = -Score::INF;

    let mut pv = PrincipalVariation::new_const();

    'iterative_deepening: for depth in 1 ..= MAX_PLY as u8 {
        td.depth = depth;
        score = if depth < 7 {
            pvs::<R, RootNode, TH>(td, state, depth, -Score::INF, Score::INF)
        } else {
            aspiration::<R, TH>(td, state, depth, score)
        };

        td.best_move = td.pvs[depth as usize].line[0];

        if td.is_aborted() {
            break 'iterative_deepening;
        }
    }

    score
}

pub fn aspiration<const R: RuleKind, TH: ThreadType>(
    td: &mut ThreadData<TH>,
    state: &mut GameState,
    max_depth: Depth,
    prev_score: Score,
) -> Score {
    let mut alpha = -Score::INF;
    let mut beta = Score::INF;
    let mut delta = ASPIRATION_WINDOW_BASE_DELTA + prev_score / 1000;
    let mut depth = max_depth;

    if max_depth >= 4 {
        alpha = alpha.max(prev_score - delta);
        beta = beta.min(prev_score + delta);
    }

    loop {
        let score = pvs::<R, RootNode, TH>(td, state, depth, alpha, beta);

        if td.is_aborted() {
            return score;
        }

        if score <= alpha { // fail-low
            beta = (alpha + beta) / 2;
            alpha = (alpha - delta).max(-Score::WIN);
            depth = max_depth;
        } else if score >= beta { // fail-high
            beta = (beta + delta).min(Score::WIN);
            depth -= 1;
        } else { // expected
            return score;
        }

        delta += delta / 2;
    }
}

pub fn pvs<const R: RuleKind, NT: NodeType, TH: ThreadType>(
    td: &mut ThreadData<TH>,
    state: &mut GameState,
    mut depth_left: Depth,
    mut alpha: i16,
    mut beta: i16,
) -> Score {
    if let &Some(pos) = state.board.patterns.unchecked_five_pos
        .access(state.board.player_color)
    { // immediate win
        return Score::win_in(td.ply)
    }

    if let &Some(pos) = state.board.patterns.unchecked_five_pos
        .access(!state.board.player_color)
    { // defend immediate win
        if state.board.player_color == Color::Black
            && state.board.patterns.forbidden_field.is_hot(pos)
        { // trapped
            return Score::lose_in(td.ply + 2)
        }

        td.push_ply_mut(state.movegen_window);
        state.set_mut(pos);

        return -pvs::<R, NT::NextType, TH>(td, state, depth_left, -beta, -alpha);
    }

    if td.config.draw_condition.is_some_and(|depth|
        state.history.len() + 1 >= depth as usize
    )
        || state.board.stones == pos::U8_BOARD_SIZE
    {
        return Score::DRAW;
    }

    if !NT::IS_ROOT {
        if td.ply >= MAX_PLY {
            return HeuristicEvaluator.eval_value(&state.board);
        }

        alpha = alpha.max(Score::win_in(td.ply));
        beta = beta.min(Score::lose_in(td.ply));
        if alpha >= beta { // mate distance pruning
            return alpha;
        }
    }

    if TH::IS_MAIN
        && td.should_check_limit()
        && td.search_limit_exceeded()
    { // search limit exceeded
        td.set_aborted_mut();
        return 0;
    } else if td.is_aborted() {
        return 0;
    }

    let mut static_eval = HeuristicEvaluator.eval_value(&state.board);
    let mut tt_move = MaybePos::NONE;
    let mut tt_pv = false;

    if let Some(entry) = td.tt.probe(state.board.hash_key) {
        let score_kind = entry.tt_flag.score_kind();
        let endgame_flag = entry.tt_flag.endgame_flag();
        tt_pv = entry.tt_flag.is_pv();

        if !(endgame_flag == EndgameFlag::Win || endgame_flag == EndgameFlag::Lose)
        { // endgame-tt-hit
            return entry.score;
        }

        if match score_kind { // tt-hit
            ScoreKind::Lower => entry.score >= beta,
            ScoreKind::Upper => entry.score <= alpha,
            ScoreKind::Exact => true,
        } {
            return entry.score;
        }

        if !match score_kind { // load tt-score
            ScoreKind::Lower => static_eval > entry.score,
            ScoreKind::Upper => static_eval < entry.score,
            ScoreKind::Exact => true
        } {
            static_eval = entry.score;
        }
    }

    td.ss[td.ply].on_pv = NT::IS_PV || tt_pv;

    if depth_left == 0 {
        return vcf_search(td, state, td.config.max_vcf_depth)
            .unwrap_or(static_eval);
    }

    let mut score_kind = ScoreKind::Upper;
    let mut best_score = i16::MIN;
    let mut best_move = tt_move;

    let mut move_picker = MovePicker::new(tt_move, td.killers[td.ply], td.counters[td.ply]);

    let mut full_window = true;
    'position_search: while let Some((pos, move_score)) = move_picker.next(state) {
        if !state.board.is_legal_move(pos) {
            continue;
        }

        td.tt.prefetch(state.board.hash_key.set(state.board.player_color, pos));

        td.push_ply_mut(state.movegen_window);
        state.set_mut(pos);

        let score = if full_window { // full-window search
            -pvs::<R, NT::NextType, TH>(td, state, depth_left - 1, -beta, -alpha)
        } else { // null-window search
            let mut score = -pvs::<R, OffPVNode, TH>(td, state, depth_left - 1, -alpha - 1, -alpha);

            if score > alpha { // null-window failed, full-window search
                score = -pvs::<R, PVNode, TH>(td, state, depth_left - 1, -beta, -alpha);
            }

            score
        };

        full_window = false;

        let movegen_window = td.pop_ply_mut();
        state.unset_mut(movegen_window);

        if score > best_score {
            best_score = score;
            best_move = pos.into();

            if score > alpha { // improve alpha
                alpha = score;
            }

            if alpha >= beta { // beta cutoff
                break 'position_search;
            }
        }

        if NT::IS_PV { // update pv-line
            let tails = td.pvs[td.ply].clone();
            let pv = &mut td.pvs[td.ply - 1];
            pv.load(pos.into(), tails);
        }
    }

    let score_kind = if best_score >= beta {
        ScoreKind::Lower
    } else if true {
        ScoreKind::Exact
    } else {
        ScoreKind::Upper
    };

    td.tt.store_mut(
        state.board.hash_key,
        best_move,
        score_kind,
        EndgameFlag::Unknown,
        depth_left,
        static_eval,
        best_score,
        NT::IS_PV
    );

    best_score
}
