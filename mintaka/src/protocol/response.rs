use crate::principal_variation::PrincipalVariation;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::value::{Depth, Score};
use std::fmt::Debug;

#[derive(Debug)]
pub enum Response {
    Info(String),
    Warning(String),
    Error(String),

    Status(Box<Status>),
    Pv(Pos, PrincipalVariation),
    BestMove(Pos, Score),
}

#[derive(Debug)]
pub struct Status {
    pub nps: f64,
    pub total_nodes_in_1k: usize,
    pub hash_usage: f64,
    pub best_moves: Vec<(Pos, Score)>,
    pub depth: Depth,
}
