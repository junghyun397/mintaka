use crate::memo::transposition_table::TranspositionTable;
use crate::principal_variation::PrincipalVariation;
use crate::value::{Depth, Score};
use rusty_renju::board::Board;
use rusty_renju::memo::slice_pattern_memo::SlicePatternMemo;
use std::marker::ConstParamTy;

//noinspection RsUnresolvedPath
#[derive(ConstParamTy, Eq, PartialEq)]
pub struct NodeType {
    pub is_root: bool,
    pub is_pv: bool,
}

impl NodeType {

    pub const ROOT: NodeType = NodeType {
        is_root: true,
        is_pv: true,
    };

    pub const PV: NodeType = NodeType {
        is_root: false,
        is_pv: true,
    };

    pub const OFF_PV: NodeType = NodeType {
        is_root: false,
        is_pv: false,
    };

    pub const CHECK: NodeType = NodeType {
        is_root: true,
        is_pv: false,
    };

}

//noinspection RsUnresolvedPath
#[derive(ConstParamTy, Eq, PartialEq)]
pub enum ThreadType {
    MainThread, SubThread
}

pub fn iterative_deepening<const Th: ThreadType>(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo, board: &mut Board,
) {
    let mut pv = PrincipalVariation::default();

    let max_depth = 0;
    for depth in 0 ..= max_depth {
    }
}

pub fn aspiration_search<const Th: ThreadType>(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo, board: &mut Board,
    pv: &mut PrincipalVariation
) {
    todo!()
}

pub fn search<const Nt: NodeType>(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo, board: &mut Board,
    pv: &mut PrincipalVariation, mut depth: Depth, mut alpha: Score, mut beta: Score,
) -> Score {
    0
}
