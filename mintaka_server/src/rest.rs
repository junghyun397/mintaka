use crate::app_state::{AppError, AppState};
use crate::session::SessionKey;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{sse, Sse};
use axum::Json;
use futures_util::TryFutureExt;
use mintaka::config::Config;
use mintaka::game_agent::BestMove;
use mintaka::protocol::command::Command;
use mintaka::protocol::message::GameResult;
use rusty_renju::board::Board;
use rusty_renju::history::History;
use serde::Serialize;
use std::convert::Infallible;
use std::num::NonZeroU32;
use std::sync::Arc;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Serialize)]
pub struct Health {
    available_workers: usize,
}

impl Into<(StatusCode, String)> for AppError {
    fn into(self) -> (StatusCode, String) {
        match self {
            AppError::InvalidSessionId => (StatusCode::BAD_REQUEST, "invalid session id".to_string()),
            AppError::SessionNotFound => (StatusCode::NOT_FOUND, "session not found".to_string()),
            AppError::SessionInComputing => (StatusCode::CONFLICT, "session in computing".to_string()),
            AppError::SessionIdle => (StatusCode::CONFLICT, "session idle".to_string()),
            AppError::StreamAcquired => (StatusCode::CONFLICT, "response stream acquired".to_string()),
            AppError::StreamNotAcquired => (StatusCode::CONFLICT, "response stream not acquired".to_string()),
            AppError::SessionNeverLaunched => (StatusCode::NO_CONTENT, "session never launched".to_string()),
            AppError::InternalError(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        }
    }
}

pub async fn status(
    State(state): State<Arc<AppState>>
) -> Json<Health> {
    Json(Health {
        available_workers: state.available_workers(),
    })
}

pub async fn new_session(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<SessionKey>) {
    let config = Config::default();
    let board = Board::default();
    let history = History::default();

    let session_key = state.new_session(config, board, history);

    (StatusCode::CREATED, Json(session_key.into()))
}

pub async fn command_session(
    Path(sid): Path<String>,
    State(state): State<Arc<AppState>>
) -> anyhow::Result<(StatusCode, Json<Option<GameResult>>), (StatusCode, String)> {
    let session_key = sid.parse::<SessionKey>().map_err(Into::into)?;

    let command = Command::Workers(NonZeroU32::new(1).unwrap());

    state.command_session(session_key, command)
        .map_err(Into::into)
        .map(|result| (StatusCode::ACCEPTED, Json(result)))
}

pub async fn launch_session(
    Path(sid): Path<String>,
    State(state): State<Arc<AppState>>
) -> anyhow::Result<StatusCode, (StatusCode, String)> {
    let session_key = sid.parse::<SessionKey>().map_err(Into::into)?;

    state.launch_session(session_key).await
        .map(|_| StatusCode::OK)
        .map_err(Into::into)
}

pub async fn subscribe_session_response(
    Path(sid): Path<String>,
    State(state): State<Arc<AppState>>
) -> anyhow::Result<Sse<UnboundedReceiverStream<Result<sse::Event, Infallible>>>, (StatusCode, String)> {
    let session_key = sid.parse::<SessionKey>().map_err(Into::into)?;

    todo!()
}

pub async fn get_session_result(
    Path(sid): Path<String>,
    State(state): State<Arc<AppState>>
) -> anyhow::Result<(StatusCode, Json<BestMove>), (StatusCode, String)> {
    let session_key = sid.parse::<SessionKey>().map_err(Into::into)?;

    state.get_session_result(session_key)
        .map_err(Into::into)
        .map(|result| (StatusCode::OK, Json(result)))
}

pub async fn abort_session(
    Path(sid): Path<String>,
    State(state): State<Arc<AppState>>
) -> anyhow::Result<StatusCode, (StatusCode, String)> {
    let session_key = sid.parse::<SessionKey>().map_err(Into::into)?;

    state.get_session_result(session_key)
        .map_err(Into::into)
        .map(|_| StatusCode::NO_CONTENT)
}

pub async fn destroy_session(
    Path(sid): Path<String>,
    State(state): State<Arc<AppState>>
) -> anyhow::Result<StatusCode, (StatusCode, String)> {
    let session_key = sid.parse::<SessionKey>().map_err(Into::into)?;

    state.destroy_session(session_key)
        .map_err(Into::into)
        .map(|_| StatusCode::NO_CONTENT)
}

pub async fn hibernate_session(
    Path(sid): Path<String>,
    State(state): State<Arc<AppState>>
) -> anyhow::Result<StatusCode, (StatusCode, String)> {
    let session_key = sid.parse::<SessionKey>().map_err(Into::into)?;

    state.hibernate_session(session_key).await
        .map_err(Into::into)
        .map(|_| StatusCode::OK)
}

pub async fn wakeup_session(
    Path(sid): Path<String>,
    State(state): State<Arc<AppState>>
) -> anyhow::Result<StatusCode, (StatusCode, String)> {
    let session_key = sid.parse::<SessionKey>().map_err(Into::into)?;

    state.wakeup_session(session_key).await
        .map_err(Into::into)
        .map(|_| StatusCode::OK)
}
