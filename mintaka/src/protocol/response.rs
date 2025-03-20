use crate::principal_variation::PrincipalVariation;
use rusty_renju::notation::pos::Pos;

pub enum Response {
    Info(String),
    Warning(String),
    Error(String),

    Status {
        eval: f32,
        total_nodes_in_1k: usize,
        best_moves: Vec<(Pos, f32)>,
        hash_usage: f32,
    },
    Pv(Vec<(Pos, PrincipalVariation)>),
    BestMove(Pos, f32),
}
