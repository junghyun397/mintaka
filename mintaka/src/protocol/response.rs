use crate::principal_variation::PrincipalVariation;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::value::Score;
use std::time::Duration;

pub enum Response {
    Info(String),
    Warning(String),
    Error(String),

    Begins {
        workers: usize,
        running_time: Duration,
        tt_size_in_kib: usize,
    },
    Status {
        eval: f32,
        total_nodes_in_1k: usize,
        best_moves: Vec<(Pos, Score)>,
        hash_usage: f32,
    },
    Pv(Vec<(Pos, PrincipalVariation)>),
    BestMove {
        best_move: Pos,
        score: Score,
        total_nodes_in_1k: usize,
        time_elapsed: Duration,
    },
}
