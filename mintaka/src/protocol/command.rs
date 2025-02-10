use crate::config::Config;
use rusty_renju::board::Board;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::rule::RuleKind;

pub enum Command {
    Init(Box<Init>),
    Status,
    Board,

    Go,
    Switch(Color),
    Move(Pos),
    UnMove,
    Abort,
}

pub struct Init {
    pub rule_kind: RuleKind,
    pub color: Color,
    pub config: Config,
    pub board: Option<Board>,
}
