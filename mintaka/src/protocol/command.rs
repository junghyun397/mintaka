use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::utils::byte_size::ByteSize;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[typeshare::typeshare]
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum Command {
    Clear,
    #[typeshare(skip)]
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
    TurnTime(Duration),
    IncrementTime(Duration),
    TotalTime(Duration),
    ConsumeTime(Duration),
    Pondering(bool),
    MaxNodes {
        #[typeshare(serialized_as = "number")]
        in_1k: u64
    },
    Workers(u32),
    MaxMemory(ByteSize),
    Rule(RuleKind),
}
