use crate::principal_variation::PrincipalVariation;
use rusty_renju::hash_key::HashKey;
use rusty_renju::impl_debug_from_display;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::score::Score;
use std::fmt::Display;
use std::time::Duration;

// typeshare-cli does not read `cfg_attr(..., serde(...))`, so keep serde attrs direct.
#[cfg(feature = "serde")]
#[cfg_attr(feature = "typeshare", typeshare::typeshare)]
#[derive(Copy, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum GameResult {
    Win(Color),
    Draw,
    Full,
}

#[cfg(not(feature = "serde"))]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum GameResult {
    Win(Color),
    Draw,
    Full,
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

#[cfg_attr(feature = "typeshare", typeshare::typeshare)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct BestMove {
    pub position_hash: HashKey,
    pub best_move: MaybePos,
    pub score: Score,
    pub selective_depth: u32,
    pub total_nodes_in_1k: u32,
    pub pv: PrincipalVariation,
    pub time_elapsed: Duration,
}
