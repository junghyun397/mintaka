use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::rule::RuleKind;
use std::num::NonZeroUsize;
use std::time::Duration;

pub enum Command {
    Load(Box<(Board, History)>),
    Play(MaybePos),
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
        player_moves: Vec<Pos>,
        opponent_moves: Vec<Pos>,
    },
    TotalTime(Duration),
    TurnTime(Duration),
    IncrementTime(Duration),
    MaxNodes { in_1k: usize },
    Workers(NonZeroUsize),
    MaxMemory { in_kib: usize },
    Rule(RuleKind),
}
