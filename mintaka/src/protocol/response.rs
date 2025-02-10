use rusty_renju::notation::pos::Pos;

#[derive(Debug)]
pub enum Response {
    Info(String),
    Warning(String),
    Error(String),

    Status(Box<Status>),
    Board(String),

    BestMove(Pos),
    Switched,
    Aborted,
}

#[derive(Debug)]
pub struct Status {
    pub nps: f64,
}
