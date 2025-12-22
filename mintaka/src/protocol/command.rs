use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::utils::byte_size::ByteSize;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::time::Duration;

use crate::config::Config;
use crate::state::GameState;
#[allow(unused_imports)]
use rusty_renju::utils::lang::DurationSchema;

#[typeshare::typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum Command {
    Clear,
    Load(Box<GameState>),
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
    TurnTime(
        #[typeshare(serialized_as = "DurationSchema")]
        Duration
    ),
    IncrementTime(
        #[typeshare(serialized_as = "DurationSchema")]
        Duration
    ),
    TotalTime(
        #[typeshare(serialized_as = "DurationSchema")]
        Duration
    ),
    ConsumeTime(
        #[typeshare(serialized_as = "DurationSchema")]
        Duration
    ),
    Pondering(bool),
    MaxNodes {
        #[typeshare(serialized_as = "number")]
        in_1k: u64
    },
    Workers(u32),
    MaxMemory(ByteSize),
    Rule(RuleKind),
    Config(Config)
}

impl Command {

    pub fn to_brief_debug(&self) -> String {
        match self {
            Self::Load(_) => "Load".to_string(),
            _ => format!("{:?}", self)
        }
    }

}
