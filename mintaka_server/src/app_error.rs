use mintaka::game_agent::GameError;
use std::fmt::{Debug, Display};
use std::io::Error;

#[derive(Debug)]
pub enum AppError {
    InvalidSessionId,
    SessionNotFound,
    SessionInComputing,
    SessionIdle,
    SessionNeverLaunched,
    StreamAcquired,
    StreamNotAcquired,
    GameError(GameError),
    InternalError(String),
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<Error> for AppError {
    fn from(e: Error) -> Self {
        Self::InternalError(e.to_string())
    }
}
