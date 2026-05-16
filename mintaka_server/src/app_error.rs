use mintaka::game_agent::GameError;
use std::fmt::{Debug, Display};

#[derive(Debug)]
pub enum AppError {
    Unauthorized,
    InvalidSessionId,
    InvalidConfig,
    SessionNotFound,
    SessionInComputing,
    SessionIdle,
    SessionNeverLaunched,
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
            Self::Unauthorized => write!(f, "UNAUTHORIZED"),
            Self::InvalidSessionId => write!(f, "INVALID_SESSION_ID"),
            Self::InvalidConfig => write!(f, "INVALID_CONFIG"),
            Self::SessionNotFound => write!(f, "SESSION_NOT_FOUND"),
            Self::SessionInComputing => write!(f, "SESSION_IN_COMPUTING"),
            Self::SessionIdle => write!(f, "SESSION_IDLE"),
            Self::SessionNeverLaunched => write!(f, "SESSION_NEVER_LAUNCHED"),
            Self::SessionFileAlreadyExists => write!(f, "SESSION_FILE_ALREADY_EXISTS"),
            Self::SessionFileNotFound => write!(f, "SESSION_FILE_NOT_FOUND"),
            Self::MemoryAcquireTimeout => write!(f, "MEMORY_ACQUIRE_TIMEOUT"),
            Self::WorkerAcquireTimeout => write!(f, "WORKER_ACQUIRE_TIMEOUT"),
            Self::GameError(err) => match err {
                GameError::InvalidConfig => write!(f, "INVALID_CONFIG"),
                GameError::HashMismatch => write!(f, "HASH_MISMATCH"),
                GameError::StoneAlreadyExist => write!(f, "STONE_ALREADY_EXIST"),
                GameError::StoneDoesNotExist => write!(f, "STONE_DOES_NOT_EXIST"),
                GameError::StoneColorMismatch => write!(f, "STONE_COLOR_MISMATCH"),
                GameError::ForbiddenMove => write!(f, "FORBIDDEN_MOVE"),
                GameError::NoHistoryToUndo => write!(f, "NO_HISTORY_TO_UNDO"),
                GameError::NoTimeManagement => write!(f, "NO_TIME_MANAGEMENT"),
            },
            Self::InternalError(_) => write!(f, "INTERNAL_ERROR"),
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
