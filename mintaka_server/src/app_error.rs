use mintaka::game_agent::GameError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("UNAUTHORIZED")]
    Unauthorized,
    #[error("INVALID_SESSION_ID")]
    InvalidSessionId,
    #[error("INVALID_CONFIG")]
    InvalidConfig,
    #[error("SESSION_NOT_FOUND")]
    SessionNotFound,
    #[error("SESSION_IN_COMPUTING")]
    SessionInComputing,
    #[error("SESSION_IDLE")]
    SessionIdle,
    #[error("SESSION_NEVER_LAUNCHED")]
    SessionNeverLaunched,
    #[error("SESSION_FILE_ALREADY_EXISTS")]
    SessionFileAlreadyExists,
    #[error("SESSION_FILE_NOT_FOUND")]
    SessionFileNotFound,
    #[error("MEMORY_ACQUIRE_TIMEOUT")]
    MemoryAcquireTimeout,
    #[error("WORKER_ACQUIRE_TIMEOUT")]
    WorkerAcquireTimeout,
    #[error("{}", game_error_code(.0))]
    GameError(#[from] GameError),
    #[error("INTERNAL_ERROR")]
    InternalError(String),
}

fn game_error_code(error: &GameError) -> &'static str {
    match error {
        GameError::HashMismatch => "HASH_MISMATCH",
        GameError::StoneAlreadyExist => "STONE_ALREADY_EXIST",
        GameError::StoneDoesNotExist => "STONE_DOES_NOT_EXIST",
        GameError::StoneColorMismatch => "STONE_COLOR_MISMATCH",
        GameError::ForbiddenMove => "FORBIDDEN_MOVE",
        GameError::NoHistoryToUndo => "NO_HISTORY_TO_UNDO",
        GameError::NoTimeManagement => "NO_TIME_MANAGEMENT",
    }
}

impl AppError {
    pub fn from_general_error(error: impl std::error::Error) -> Self {
        Self::InternalError(error.to_string())
    }
}
