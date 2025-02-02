use crate::memo::transposition_table::TranspositionTable;
use crate::principal_variation::PrincipalVariation;
use crate::thread_data::ThreadData;
use crate::value::{Depth, Score};
use rusty_renju::board::Board;
use std::marker::ConstParamTy;

pub trait NodeType {

    const IS_ROOT: bool;
    const IS_PV: bool;

    type NextType: NodeType;

}

struct RootNode {} impl NodeType for RootNode {
    const IS_ROOT: bool = true;
    const IS_PV: bool = true;
    type NextType = PVNode;
}

struct PVNode {} impl NodeType for PVNode {
    const IS_ROOT: bool = false;
    const IS_PV: bool = true;
    type NextType = Self;
}

struct OffPVNode {} impl NodeType for OffPVNode {
    const IS_ROOT: bool = false;
    const IS_PV: bool = false;
    type NextType = Self;
}

//noinspection RsUnresolvedPath
#[derive(ConstParamTy, Eq, PartialEq)]
pub enum ThreadType {
    MainThread, SubThread
}

pub fn iterative_deepening<const TH: ThreadType>(
    tt: &TranspositionTable, board: &mut Board,
    td: &mut ThreadData
) {
    let mut pv = PrincipalVariation::default();

    let max_depth: Depth = 0;
    for depth in 0 ..= max_depth {
    }
}

pub fn aspiration_search<const TH: ThreadType>(
    tt: &TranspositionTable, board: &mut Board,
    pv: &mut PrincipalVariation,
    td: &mut ThreadData
) {
    todo!()
}

pub fn negamax<NT: NodeType>(
    tt: &TranspositionTable, board: &mut Board,
    pv: &mut PrincipalVariation,
    td: &mut ThreadData,
    mut depth: Depth, mut alpha: Score, mut beta: Score,
) -> Score {
    if NT::IS_ROOT {
    }

    if NT::IS_PV {
    }

    let beta = -negamax::<NT::NextType>(tt, board, pv, td, depth, alpha, beta);

    0
}
