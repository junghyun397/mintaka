use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::rule::RuleKind;

pub enum Command {
    Init(RuleKind, Color),
    Set(Pos),
    Unset(Pos),
}
