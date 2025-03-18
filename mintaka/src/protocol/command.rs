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
    Switch,
    BatchSet {
        black_stones: Box<[Pos]>,
        white_stones: Box<[Pos]>,
        player_color: Color,
    },
    TotalTime(Duration),
    TurnTime(Duration),
    IncrementTime(Duration),
    MaxMemory { in_kib: usize },
    Rule(RuleKind),
}
