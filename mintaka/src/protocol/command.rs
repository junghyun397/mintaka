use rusty_renju::history::Action;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::rule::RuleKind;
use std::time::Duration;

pub enum Command {
    Play(Action),
    Set {
        pos: Pos,
        color: Color,
    },
    Unset {
        pos: Pos,
        color: Color,
    },
    Undo,
    TotalTime(Duration),
    TurnTime(Duration),
    Rule(RuleKind),
}
