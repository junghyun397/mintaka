use rusty_renju::board::Board;
use rusty_renju::history::{Action, History};
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::rule::RuleKind;
use std::time::Duration;

pub enum Command {
    Load(Box<Board>, History),
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
    BatchSet {
        player_stones: Vec<Pos>,
        opponent_stones: Vec<Pos>,
    },
    TotalTime(Duration),
    TurnTime(Duration),
    IncrementTime(Duration),
    MaxMemory { in_kib: usize },
    Rule(RuleKind),
}
