use crate::config::Config;
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::utils::byte_size::ByteSize;
#[allow(unused_imports)]
use rusty_renju::utils::lang::DurationSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::time::Duration;
#[cfg(feature = "typeshare")]
use typeshare::typeshare;

#[cfg_attr(feature = "typeshare", typeshare)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct CompactGameState {
    pub board: Board,
    pub history: History,
}

#[cfg_attr(feature = "typeshare", typeshare(serialized_as = "CommandSchema"))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "content"))]
#[cfg_attr(feature = "serde", serde_with::skip_serializing_none)]
#[derive(Debug, Clone)]
pub enum Command {
    Clear,
    Load(Box<CompactGameState>),
    Sync(Box<CompactGameState>),
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
    MaxNodes { in_1k: u32 },
    Workers(u32),
    MaxMemory(ByteSize),
    Rule(RuleKind),
    Config(Config)
}

#[cfg(any())]
mod typeshare_workaround {
    use super::*;
    #[typeshare]
    #[derive(Serialize, Deserialize)]
    #[serde(tag = "type", content = "content")]
    pub enum CommandSchema {
        Clear,
        Load(Box<CompactGameState>),
        Sync(Box<CompactGameState>),
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
        TurnTime(#[typeshare(serialized_as = "DurationSchema")] Duration),
        IncrementTime(#[typeshare(serialized_as = "DurationSchema")] Duration),
        TotalTime(#[typeshare(serialized_as = "DurationSchema")] Duration),
        ConsumeTime(#[typeshare(serialized_as = "DurationSchema")] Duration),
        Pondering(bool),
        MaxNodes { in_1k: u32 },
        Workers(u32),
        MaxMemory(ByteSize),
        Rule(RuleKind),
        Config(Config)
    }
}

impl Command {

    pub fn to_brief_debug(&self) -> String {
        match self {
            Self::Load(_) => "Load".to_string(),
            _ => format!("{:?}", self)
        }
    }

}
