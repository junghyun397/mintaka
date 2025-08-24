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

impl Error for AppError {}

impl AppError {

    pub fn from_general_error<T: Error>(err: T) -> Self {
        Self::InternalError(err.to_string())
    }

}
