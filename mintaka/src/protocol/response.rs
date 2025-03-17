use crate::principal_variation::PrincipalVariation;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::value::Score;
use std::fmt::Debug;

#[derive(Debug)]
pub enum Response {
    Info(String),
    Warning(String),
    Error(String),

    Status {
        total_nodes_in_1k: usize,
        hash_usage: f64,
        best_moves: Vec<(Pos, Score)>,
    },
    Pv(Pos, PrincipalVariation),
    BestMove(Pos, Score),
}
