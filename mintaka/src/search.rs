use crate::endgame::vcf_search::vcf_search;
use crate::eval::evaluator::Evaluator;
use crate::eval::heuristic_evaluator::HeuristicEvaluator;
use crate::game_state::GameState;
use crate::memo::tt_entry::{EndgameFlag, ScoreKind};
use crate::movegen::move_picker::MovePicker;
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use crate::value::{ASPIRATION_INITIAL_DELTA, MAX_PLY};
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
    td: &mut ThreadData<TH>,
    mut state: GameState,
) -> Score {
    let mut score: Score = 0;

    'iterative_deepening: for depth in 1 ..= td.config.max_depth {
        td.depth = depth;

        score = if depth < 7 || true { // todo: debug
            pvs::<R, RootNode, TH>(td, &mut state, depth, -Score::INF, Score::INF)
        } else {
            aspiration::<R, TH>(td, &mut state, depth, score)
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
    if state.board.stones as usize >= td.config.draw_condition {
        return Score::DRAW;
    }

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
            return Score::lose_in(td.ply + 2)
        }

        td.push_ply_mut();
        let movegen_window = state.movegen_window;
        state.set_mut(pos);

        let score = -pvs::<R, NT, TH>(td, state, depth_left, -beta, -alpha);

        state.unset_mut(movegen_window);
        td.pop_ply_mut();

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

    if TH::IS_MAIN
        && td.should_check_limit()
        && td.search_limit_exceeded()
    {
        td.set_aborted_mut();
        return Score::DRAW;
    }

    if td.is_aborted() {
        return Score::DRAW;
    }

    // clear pv-line
    td.pvs[td.ply].clear();

    let mut static_eval: Score;
    let mut tt_move: MaybePos;
    let mut tt_pv: bool;
    let mut tt_endgame_flag: EndgameFlag;

    if let Some(entry) = td.tt.probe(state.board.hash_key) && false { // todo: debug
        tt_endgame_flag = entry.tt_flag.endgame_flag();
        if tt_endgame_flag == EndgameFlag::Win || tt_endgame_flag == EndgameFlag::Lose
        { // endgame tt-hit
            return entry.score as Score;
        }

        static_eval = entry.eval as Score;
        tt_move = entry.best_move;
        tt_pv = entry.tt_flag.is_pv();

        if !NT::IS_PV && match entry.tt_flag.score_kind() { // tt-cutoff
            ScoreKind::LowerBound => entry.score as Score >= beta,
            ScoreKind::UpperBound => entry.score as Score <= alpha,
            ScoreKind::Exact => true,
        } {
            return entry.score as Score;
        }

        static_eval = entry.score as Score;
    } else {
        tt_endgame_flag = EndgameFlag::Unknown;
        static_eval = HeuristicEvaluator.eval_value(&state.board);
        tt_move = MaybePos::NONE;
        tt_pv = false;
    }

    if td.ply + 1 >= MAX_PLY || depth_left == 0 {
        return vcf_search(td, td.config.max_vcf_depth, state, alpha, beta)
            .unwrap_or(static_eval);
    }

    let original_alpha = alpha;

    let mut best_score = -Score::INF;
    let mut best_move = MaybePos::NONE;

    let mut move_picker = MovePicker::new(tt_move, td.killers[td.ply]);

    td.ss[td.ply].static_eval = static_eval;
    td.ss[td.ply].on_pv = NT::IS_PV || tt_pv;

    let mut move_seq = 0;
    'position_search: while let Some((pos, _)) = move_picker.next(state) {
        if !state.board.is_legal_move(pos) {
            continue;
        }

        td.tt.prefetch(state.board.hash_key.set(state.board.player_color, pos));

        td.push_ply_mut();
        let movegen_window = state.movegen_window;
        state.set_mut(pos);

        move_seq += 1;

        let score = if move_seq == 1 { // full-window search
            -pvs::<R, NT::NextType, TH>(td, state, depth_left - 1, -beta, -alpha)
        } else { // zero-window search
            let mut score = -pvs::<R, OffPVNode, TH>(td, state, depth_left - 1, -alpha - 1, -alpha);

            if score > alpha { // zero-window failed, full-window search
                score = -pvs::<R, PVNode, TH>(td, state, depth_left - 1, -beta, -alpha);
            }

            score
        };

        state.unset_mut(movegen_window);
        td.pop_ply_mut();

        best_score = best_score.max(score);

        if score > alpha { // improve alpha
            best_move = pos.into();
            alpha = score;

            if NT::IS_PV { // update parent node pv
                if NT::IS_ROOT {
                    td.pvs[0].line[0] = pos.into();
                } else {
                    let sub_pv = td.pvs[td.ply];
                    td.pvs[td.ply - 1].load(pos.into(), sub_pv);
                }
            }
        }

        if alpha >= beta { // beta cutoff
            td.ss[td.ply].cutoffs += 1;
            td.insert_killer_move_mut(pos);
            break 'position_search;
        }
    }

    if td.is_aborted() {
        return Score::DRAW;
    }

    let score_kind = if best_score >= beta {
        ScoreKind::LowerBound
    } else if best_score > original_alpha {
        ScoreKind::Exact
    } else {
        ScoreKind::UpperBound
    };

    td.tt.store(
        state.board.hash_key,
        best_move,
        score_kind,
        tt_endgame_flag,
        td.ply as u8,
        static_eval,
        best_score,
        NT::IS_PV
    );

    if NT::IS_ROOT {
        td.best_move = best_move;
    }

    best_score
}
