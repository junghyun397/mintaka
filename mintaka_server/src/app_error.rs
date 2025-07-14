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
    InternalError(String)
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
