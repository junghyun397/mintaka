use crate::game_state::GameState;
use crate::memo::tt_entry::{EndgameFlag, ScoreKind};
use crate::movegen::move_picker::MovePicker;
use crate::principal_variation::PrincipalVariation;
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::value::Depth;

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
) -> i32 {

    let mut eval: i32 = 0;
    let mut score: i32 = 0;

    let max_depth: Depth = 0;

    'iterative_deepening: for depth in 1 ..= max_depth {
        eval = pvs::<R, RootNode, TH>(td, state, depth, -i32::MAX, i32::MAX);

        if TH::IS_MAIN {
            if td.node_limit_exceeded() {
                td.set_aborted_mut();

                break 'iterative_deepening;
            }
        }

        if td.is_aborted() {
            break 'iterative_deepening;
        }
    }

    score
}

pub fn aspiration<const R: RuleKind, TH: ThreadType>(
    td: &mut ThreadData<TH>,
    state: &mut GameState,
    pv: &mut PrincipalVariation,
) {
    todo!()
}

pub fn pvs<const R: RuleKind, NT: NodeType, TH: ThreadType>(
    td: &mut ThreadData<TH>,
    state: &mut GameState,
    mut depth_left: Depth,
    mut alpha: i32,
    mut beta: i32,
) -> i32 {
    if let Some(pos) = *state.board.patterns.unchecked_five_pos.access(state.board.player_color) {
        td.best_move = pos;
        return i32::MAX;
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

    if !NT::IS_ROOT {
        if alpha >= beta {
            return alpha;
        }
    }

    let pv = PrincipalVariation::default();

    let mut eval = 0;
    let mut tt_move = MaybePos::NONE;
    let mut tt_score = 0;

    if let Some(entry) = td.tt.probe(state.board.hash_key) {
        let score_kind = entry.tt_flag.score_kind();
        let endgame_flag = entry.tt_flag.endgame_flag();
        let is_pv = entry.tt_flag.is_pv();

        match endgame_flag {
            EndgameFlag::Win => {
                return i32::MAX;
            },
            EndgameFlag::Lose => {
                return i32::MIN;
            },
            _ => {}
        }

        tt_move = entry.best_move;
        tt_score = entry.score;

        if match score_kind {
            ScoreKind::Lower => eval <= entry.score,
            ScoreKind::Upper => eval >= entry.score,
            _ => false
        } {
            eval = entry.score
        }
    }

    let mut score_kind = ScoreKind::Upper;
    let mut best_score = i32::MIN;
    let mut best_move = tt_move;

    let mut move_picker = MovePicker::new(
        td,
        None,
        None,
    );

    let mut full_window = true;
    'position_search: while let Some((pos, move_eval)) = move_picker.next(td, state) {
        let movegen_window = state.movegen_window;
        state.set(pos);

        let score = if full_window {
            -pvs::<R, NT::NextType, TH>(td, state, depth_left - 1, -beta, -alpha)
        } else {
            -pvs::<R, NT::NextType, TH>(td, state, depth_left - 1, -alpha - 1, -alpha)
        };

        state.unset(movegen_window);

        best_score = best_score.max(score);

        // alpha-cutoff
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
