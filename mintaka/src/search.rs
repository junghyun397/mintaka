use crate::endgame;
use crate::game_state::GameState;
use crate::memo::tt_entry::{EndgameFlag, ScoreKind};
use crate::movegen::move_picker::MovePicker;
use crate::principal_variation::PrincipalVariation;
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::value::{Depth, Score};

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

    let mut score: Score = Score::MIN;

    let max_depth: Depth = 0;

    'iterative_deepening: for depth in 1 ..= max_depth {
        score = if depth < 7 {
            pvs::<R, RootNode, TH>(td, state, depth, Score::MIN, Score::MAX)
        } else {
            aspiration::<R, TH>(td, state, depth, score)
        };

        // TODO: set best-move

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
    mut score: Score,
) -> Score {
    let mut delta = 5 + score / 1000;
    let mut alpha = Score::MIN;
    let mut beta = Score::MAX;
    let mut depth = max_depth;

    if max_depth >= 4 {
        alpha = alpha.max(score - delta);
        beta = beta.min(score + delta);
    }

    loop {
        score = pvs::<R, RootNode, _>(td, state, 0, alpha, beta);

        if td.is_aborted() {
            return 0;
        }

        if score <= alpha { // fail-low
            beta = (alpha + beta) / 2;
            alpha = score.saturating_sub(delta);
            depth = max_depth;
        } else if score >= beta { // fail-high
            beta = score.saturating_add(delta);
            depth -= 1;
        } else { // exact score
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
    if state.board.patterns.unchecked_five_pos.access(state.board.player_color).is_some() {
        return i16::MAX;
    }

    if let &Some(pos) = state.board.patterns.unchecked_five_pos.access(!state.board.player_color) {
        state.set(pos.into()); // forced move
        return -pvs::<R, NT, TH>(td, state, depth_left - 1, -beta, -alpha);
    }

    if td.config.draw_condition.is_some_and(|depth|
        state.board.stones + 2 > depth
    ) {
        return 0;
    }

    if TH::IS_MAIN
        && td.batch_counter.count_local_total() % 1024 == 0
        && td.search_limit_exceeded()
    {
        td.set_aborted_mut();
        return 0;
    } else if td.is_aborted() {
        return 0;
    }

    if !NT::IS_ROOT
        && alpha >= beta
    {
        return alpha;
    }

    let pv = PrincipalVariation::default();

    let mut static_eval = 0;
    let mut tt_move = MaybePos::NONE;
    let mut tt_score = 0;

    if let Some(entry) = td.tt.probe(state.board.hash_key) {
        let score_kind = entry.tt_flag.score_kind();
        let endgame_flag = entry.tt_flag.endgame_flag();
        let is_pv = entry.tt_flag.is_pv();

        match endgame_flag {
            EndgameFlag::Win => {
                return Score::MAX;
            },
            EndgameFlag::Lose => {
                return Score::MIN;
            },
            _ => {}
        }

        tt_move = entry.best_move;
        tt_score = entry.score;

        if match score_kind {
            ScoreKind::Lower => static_eval <= entry.score,
            ScoreKind::Upper => static_eval >= entry.score,
            _ => false
        } {
            static_eval = entry.score;
        }
    }

    if depth_left == 0 {
        return endgame::vcf::vcf_search(td, state, td.config.max_vcf_depth) // drop into vcf search
            .unwrap_or(static_eval);
    }

    let mut score_kind = ScoreKind::Upper;
    let mut best_score = i16::MIN;
    let mut best_move = tt_move;

    let mut move_picker = MovePicker::new(tt_move, td.search_stack.last().unwrap().killer_moves);

    let mut full_window = true;
    'position_search: while let Some((pos, move_score)) = move_picker.next(state) {
        let movegen_window = state.movegen_window;
        state.set(pos);

        let score = if full_window {
            -pvs::<R, PVNode, TH>(td, state, depth_left - 1, -beta, -alpha)
        } else {
            -pvs::<R, OffPVNode, TH>(td, state, depth_left - 1, -alpha - 1, -alpha)
        };

        full_window = false;

        state.unset(movegen_window);

        best_score = best_score.max(score);

        if score <= alpha {
            continue 'position_search;
        }

        best_move = pos.into();
        alpha = score;
        score_kind = ScoreKind::Exact;

        // beta-cutoff
        if score < beta {
            continue 'position_search;
        }

        score_kind = ScoreKind::Lower;

        break 'position_search;
    }

    if td.is_aborted() {
        return 0;
    }

    best_score
}
