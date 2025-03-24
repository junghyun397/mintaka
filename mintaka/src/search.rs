use crate::game_state::GameState;
use crate::memo::tt_entry::ScoreKind;
use crate::movegen::move_generator::generate_moves;
use crate::principal_variation::PrincipalVariation;
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::value::{Depth, Eval, Score};

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

pub fn iterative_deepening<TH: ThreadType>(
    td: &mut ThreadData<TH>,
    state: &mut GameState,
) -> (Pos, Score) {
    let mut best_move = MaybePos::NONE;
    let mut eval: Eval = 0;
    let mut score: Score = 0;

    let max_depth: Depth = 0;
    let initial_depth = 1 + td.tid as Depth % 10;

    'iterative_deepening: for depth in initial_depth ..= max_depth {
        eval = pvs::<RootNode, TH>(td, state, depth, -Score::MAX, Score::MAX);

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

    (best_move.unwrap(), score)
}

pub fn aspiration<TH: ThreadType>(
    td: &mut ThreadData<TH>,
    state: &mut GameState,
    pv: &mut PrincipalVariation,
) {
    todo!()
}

pub fn pvs<NT: NodeType, TH: ThreadType>(
    td: &mut ThreadData<TH>,
    state: &mut GameState,
    mut remaining_depth: Depth,
    mut alpha: Score,
    mut beta: Score,
) -> Score {
    let pv = PrincipalVariation::default();

    if NT::IS_ROOT {
    }

    if NT::IS_PV {
    }

    let mut eval = 0;
    let mut tt_move = MaybePos::NONE;
    let mut tt_score = 0;

    if let Some(entry) = td.tt.probe(state.board.hash_key) {
        let score_kind = entry.tt_flag.score_kind();
        let endgame_flag = entry.tt_flag.endgame_flag();
        let is_pv = entry.tt_flag.is_pv();

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
    let mut best_score = Score::MIN;
    let mut best_move = tt_move;

    let mut moves = generate_moves(&state.board, &state.movegen_window);

    let mut full_window = true;
    for pos in moves {
        let score = if full_window {
            pvs::<NT::NextType, TH>(td, state, remaining_depth - 1, -beta, -alpha)
        } else {
            pvs::<NT::NextType, TH>(td, state, remaining_depth - 1, -alpha - 1, -alpha)
        };

        best_score = best_score.max(score);

        // alpha-cutoff
        if score <= alpha {
            continue;
        }

        best_move = pos.into();
        alpha = score;
        score_kind = ScoreKind::Exact;

        // beta-cutoff
        if score < beta {
            continue;
        }

        score_kind = ScoreKind::Lower;

        break;
    }

    best_score
}
