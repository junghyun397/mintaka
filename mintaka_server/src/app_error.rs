use mintaka::game_agent::GameError;
use std::fmt::{Debug, Display};

#[derive(Debug)]
pub enum AppError {
    InvalidSessionId,
    InvalidConfig,
    SessionNotFound,
    SessionInComputing,
    SessionIdle,
    SessionNeverLaunched,
    StreamAlreadyAcquired,
    StreamNotAcquired,
    SessionFileAlreadyExists,
    SessionFileNotFound,
    MemoryAcquireTimeout,
    WorkerAcquireTimeout,
    GameError(GameError),
    InternalError(String),
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSessionId => write!(f, "invalid session id"),
            Self::InvalidConfig => write!(f, "invalid config"),
            Self::SessionNotFound => write!(f, "session not found"),
            Self::SessionInComputing => write!(f, "session is in computing"),
            Self::SessionIdle => write!(f, "session is idle"),
            Self::SessionNeverLaunched => write!(f, "session never launched"),
            Self::StreamAlreadyAcquired => write!(f, "stream is already acquired"),
            Self::StreamNotAcquired => write!(f, "stream is not acquired"),
            Self::SessionFileAlreadyExists => write!(f, "session file already exists"),
            Self::SessionFileNotFound => write!(f, "session file not found"),
            Self::MemoryAcquireTimeout => write!(f, "memory acquisition timed out"),
            Self::WorkerAcquireTimeout => write!(f, "worker acquisition timed out"),
            Self::GameError(err) => write!(f, "{err}"),
            Self::InternalError(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<GameError> for AppError {
    fn from(err: GameError) -> Self {
        match err {
            GameError::InvalidConfig => Self::InvalidConfig,
            _ => Self::GameError(err),
        }
    }
}

impl AppError {

    pub fn from_general_error(error: impl std::error::Error) -> Self {
        Self::InternalError(error.to_string())
    }

}
