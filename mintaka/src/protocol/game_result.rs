use rusty_renju::impl_debug_from_display;
use rusty_renju::notation::color::Color;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[typeshare::typeshare]
#[derive(Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum GameResult {
    Win(Color),
    Draw,
    Full
}

impl Display for GameResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameResult::Win(color) => write!(f, "{color:?} win", ),
            GameResult::Draw => write!(f, "draw"),
            GameResult::Full => write!(f, "full"),
        }
    }
}

impl_debug_from_display!(GameResult);
