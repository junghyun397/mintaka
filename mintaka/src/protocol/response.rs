use rusty_renju::notation::pos::Pos;

pub enum Response {
    Move(Pos),
    Status(String),
}
