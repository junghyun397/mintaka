use crate::principal_variation::PrincipalVariation;
use rusty_renju::impl_debug_from_display;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::value::Score;
use rusty_renju::utils::byte_size::ByteSize;
use std::fmt::Display;
use std::time::Duration;

pub enum Response {
    Info(String),
    Warning(String),
    Error(String),

    Begins {
        workers: usize,
        running_time: Duration,
        tt_size: ByteSize,
    },
    Status {
        eval: f32,
        total_nodes_in_1k: usize,
        best_moves: Vec<(Pos, Score)>,
        hash_usage: f32,
    },
    Pv(Vec<(Pos, PrincipalVariation)>),
    BestMove {
        best_move: Pos,
        score: Score,
        total_nodes_in_1k: usize,
        time_elapsed: Duration,
    },
    Finished(GameResult),
}

#[derive(Eq, PartialEq)]
pub enum GameResult {
    Win(Color),
    Draw,
    Full
}

impl Display for GameResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameResult::Win(color) => write!(f, "{:?} win", color),
            GameResult::Draw => write!(f, "draw"),
            GameResult::Full => write!(f, "full"),
        }
    }
}

impl_debug_from_display!(GameResult);
