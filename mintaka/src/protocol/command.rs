use crate::game_state::GameStateData;
use rusty_renju::hash_key::HashKey;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::utils::byte_size::ByteSize;
use std::fmt::Debug;

#[cfg(feature = "serde")]
#[cfg_attr(feature = "typeshare", typeshare::typeshare)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "content")]
#[serde_with::skip_serializing_none]
pub enum Command {
    Clear,
    Init(Box<GameStateData>),
    Sync(Box<GameStateData>),
    Play {
        hash: HashKey,
        pos: MaybePos,
        draw_condition: Option<u32>,
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
    RebuildTT(ByteSize),
}

#[cfg(not(feature = "serde"))]
#[derive(Debug, Clone)]
pub enum Command {
    Clear,
    Init(Box<GameStateData>),
    Sync(Box<GameStateData>),
    Play {
        hash: HashKey,
        pos: MaybePos,
        draw_condition: Option<u32>,
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
    RebuildTT(ByteSize),
}
