use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::value::Score;

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
}
