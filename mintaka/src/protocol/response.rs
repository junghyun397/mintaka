use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::value::{Depth, Score};

#[derive(Debug)]
pub enum Response {
    Info(String),
    Warning(String),
    Error(String),

    Status(Box<Status>),
    Board(String),

    BestMove(Pos, Score),
    Switched,
    Aborted,
}

#[derive(Debug)]
pub struct Status {
    pub nps: f64,
    pub total_nodes_in_1k: usize,
    pub tt_size_in_kib: usize,
    pub hash_usage: f64,
    pub best_moves: Vec<(Pos, Score)>,
    pub depth: Depth,
}
