use rusty_renju::impl_debug_from_display;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::color::Color;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[typeshare::typeshare]
#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum GameResult {
    Win(Color),
    Draw,
    Full
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

#[typeshare::typeshare]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
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
