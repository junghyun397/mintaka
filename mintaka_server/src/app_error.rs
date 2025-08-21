use mintaka::game_agent::GameError;
use std::error::Error;
use std::fmt::{Debug, Display};

#[derive(Debug)]
pub enum AppError {
    InvalidSessionId,
    SessionNotFound,
    SessionInComputing,
    SessionIdle,
    SessionNeverLaunched,
    StreamAcquired,
    StreamNotAcquired,
    SessionFileAlreadyExists,
    SessionFileNotFound,
    GameError(GameError),
    InternalError(String),
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<T: Error> From<T> for AppError {
    fn from(e: T) -> Self {
        Self::InternalError(e.to_string())
    }
}

impl Error for AppError {}
