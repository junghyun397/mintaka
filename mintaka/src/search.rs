use crate::game_state::GameState;
use crate::principal_variation::PrincipalVariation;
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use rusty_renju::notation::pos::Pos;
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
    let mut best_move = Pos::INVALID;
    let mut eval: Eval = 0;
    let mut score: Score = 0;

    let mut pv = PrincipalVariation::default();

    let max_depth: Depth = 0;
    for depth in 0 ..= max_depth {
    }

    (best_move, score)
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
    pv: &mut PrincipalVariation,
    state: &mut GameState,
    mut depth: Depth,
    mut alpha: Score,
    mut beta: Score,
) -> Score {
    if td.is_aborted() || td.limit_reached() {
        return 0;
    }

    if NT::IS_ROOT {
    }

    if NT::IS_PV {
    }

    let beta = -pvs::<NT::NextType, TH>(td, pv, state, depth, alpha, beta);

    0
}
