use rusty_renju::impl_debug_from_display;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::color::Color;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::time::Duration;
#[cfg(feature = "typeshare")]
use typeshare::typeshare;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::score::Score;
use crate::principal_variation::PrincipalVariation;

#[cfg_attr(feature = "typeshare", typeshare(serialized_as = "GameResultSchema"))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "content"))]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum GameResult {
    Win(Color),
    Draw,
    Full
}

#[cfg(any())]
mod typeshare_workaround {
    use super::*;
    #[typeshare]
    #[derive(Serialize, Deserialize)]
    #[serde(tag = "type", content = "content")]
    pub enum GameResultSchema {
        Win(Color),
        Draw,
        Full
    }
}

impl Display for GameResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameResult::Win(color) => write!(f, "{color:?} win"),
            GameResult::Draw => write!(f, "draw"),
            GameResult::Full => write!(f, "full"),
        }
    }
}

impl_debug_from_display!(GameResult);

#[cfg_attr(feature = "typeshare", typeshare)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde_with::skip_serializing_none)]
#[derive(Debug, Copy, Clone)]
pub struct CommandResult {
    pub hash_key: HashKey,
    pub result: Option<GameResult>,
}

impl CommandResult {
    pub fn finished(hash_key: HashKey, result: GameResult) -> Self {
        Self { hash_key, result: Some(result) }
    }

    pub fn hash(hash_key: HashKey) -> Self {
        Self { hash_key, result: None }
    }
}

#[cfg_attr(feature = "typeshare", typeshare::typeshare)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct BestMove {
    pub position_hash: HashKey,
    pub best_move: MaybePos,
    pub score: Score,
    pub selective_depth: u32,
    pub total_nodes_in_1k: u32,
    pub pv: PrincipalVariation,
    #[cfg_attr(feature = "typeshare", typeshare(serialized_as = "DurationSchema"))]
    pub time_elapsed: Duration,
}
