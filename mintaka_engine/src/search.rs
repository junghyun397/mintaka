use crate::memo::transposition_table::TranspositionTable;
use mintaka::board::Board;
use mintaka::memo::slice_pattern_memo::SlicePatternMemo;
use mintaka::notation::node::{Depth, Score};
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

}

pub fn iterative_deepening() {
    todo!()
}

pub fn aspiration_search<const NT: NodeType>() {
    todo!()
}

// Principal Variation Search(PVS), https://www.chessprogramming.org/Principal_Variation_Search
pub fn search<const NT: NodeType>(
    tt: &mut TranspositionTable, memo: &mut impl SlicePatternMemo,
    board: &mut Board,
    mut depth: Depth, mut alpha: Score, mut beta: Score,
) -> Score {
    0
}
