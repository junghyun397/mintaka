use crate::config::Config;
use crate::game_state::GameStateData;
use rusty_renju::hash_key::HashKey;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::utils::byte_size::ByteSize;
#[allow(unused_imports)]
use rusty_renju::utils::lang::DurationSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::time::Duration;
#[cfg(feature = "typeshare")]
use typeshare::typeshare;

#[cfg_attr(feature = "typeshare", typeshare(serialized_as = "CommandSchema"))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "content"))]
#[cfg_attr(feature = "serde", serde_with::skip_serializing_none)]
#[derive(Debug, Clone)]
pub enum Command {
    Clear,
    Init(Box<GameStateData>),
    Sync(Box<GameStateData>),
    Play {
        hash: HashKey,
        pos: MaybePos,
    },
    Set {
        hash: HashKey,
        pos: Pos,
        color: Color,
    },
    Unset {
        hash: HashKey,
        pos: Pos,
        color: Color,
    },
    Undo {
        hash: HashKey,
    },
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
        in_1k: u32,
    },
    Workers(u32),
    MaxMemory(ByteSize),
    Config(Config),
}

#[cfg(any())]
mod typeshare_workaround {
    use super::*;
    #[typeshare]
    #[derive(Serialize, Deserialize)]
    #[serde(tag = "type", content = "content")]
    pub enum CommandSchema {
        Clear,
        Init(Box<GameStateData>),
        Sync(Box<GameStateData>),
        Play {
            hash: HashKey,
            pos: MaybePos,
        },
        Set {
            hash: HashKey,
            pos: Pos,
            color: Color,
        },
        Unset {
            hash: HashKey,
            pos: Pos,
            color: Color,
        },
        Undo {
            hash: HashKey,
        },
        BatchSet {
            player_moves: Vec<Pos>,
            opponent_moves: Vec<Pos>,
        },
        TurnTime(#[typeshare(serialized_as = "DurationSchema")] Duration),
        IncrementTime(#[typeshare(serialized_as = "DurationSchema")] Duration),
        TotalTime(#[typeshare(serialized_as = "DurationSchema")] Duration),
        ConsumeTime(#[typeshare(serialized_as = "DurationSchema")] Duration),
        Pondering(bool),
        MaxNodes {
            in_1k: u32,
        },
        Workers(u32),
        MaxMemory(ByteSize),
        Config(Config),
    }
}
