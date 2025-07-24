use crate::app_state::{AppError, AppState};
use crate::session::{SessionKey, SessionResponse};
use async_stream::stream;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{sse, IntoResponse, Sse};
use axum::Json;
use futures_util::Stream;
use mintaka::config::Config;
use mintaka::protocol::command::Command;
use rusty_renju::board::Board;
use rusty_renju::history::History;
use serde::Serialize;
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::StreamExt;

#[derive(Serialize)]
pub struct Health {
    available_workers: usize,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::InvalidSessionId => (StatusCode::BAD_REQUEST, "invalid session id".to_string()),
            AppError::SessionNotFound => (StatusCode::NOT_FOUND, "session not found".to_string()),
            AppError::SessionInComputing => (StatusCode::CONFLICT, "session in computing".to_string()),
            AppError::SessionIdle => (StatusCode::CONFLICT, "session idle".to_string()),
            AppError::StreamAcquired => (StatusCode::CONFLICT, "response stream acquired".to_string()),
            AppError::StreamNotAcquired => (StatusCode::CONFLICT, "response stream not acquired".to_string()),
            AppError::SessionNeverLaunched => (StatusCode::NO_CONTENT, "session never launched".to_string()),
            AppError::SessionFileAlreadyExists => (StatusCode::CONFLICT, "internal session data already exists".to_string()),
            AppError::SessionFileNotFound => (StatusCode::NOT_FOUND, "internal session data does not exist".to_string()),
            AppError::GameError(game_error) => (StatusCode::CONFLICT, game_error.to_string()),
            AppError::InternalError(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        }.into_response()
    }
}

pub async fn status(
    State(state): State<Arc<AppState>>
) -> impl IntoResponse {
    Json(Health {
        available_workers: state.available_workers(),
    })
}

pub async fn new_session(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let config = Config::default();
    let board = Board::default();
    let history = History::default();

    let session_key = state.new_session(config, board, history);

    (StatusCode::CREATED, Json(session_key))
}

pub async fn command_session(
    Path(sid): Path<SessionKey>,
    State(state): State<Arc<AppState>>,
    Json(command): Json<Command>
) -> impl IntoResponse {
    state.command_session(sid, command.into())
        .map(|response| (StatusCode::ACCEPTED, Json(response)))
}

pub async fn launch_session(
    Path(sid): Path<SessionKey>,
    State(state): State<Arc<AppState>>
) -> impl IntoResponse {
    state.launch_session(sid).await
        .map(|computing_resource| (StatusCode::OK, Json(computing_resource)))
}

pub async fn subscribe_session_response(
    Path(sid): Path<SessionKey>,
    State(state): State<Arc<AppState>>
) -> Result<Sse<impl Stream<Item = Result<sse::Event, Infallible>>>, AppError> {
    let session_stream = state.acquire_session_stream(sid)?;
    let state = state.clone();

    let sse_stream = stream! {
        let mut receiver = session_stream;

        while let Some(session_response) = receiver.next().await {
            match session_response {
                SessionResponse::Response(response) => {
                    yield Ok(sse::Event::default()
                        .event("response")
                        .json_data(response)
                        .unwrap());
                },
                SessionResponse::BestMove(best_move) => {
                    yield Ok(sse::Event::default()
                        .event("best-move")
                        .json_data(best_move)
                        .unwrap());
                },
                SessionResponse::Terminate => {
                    yield Ok(sse::Event::default()
                        .event("terminate"));

                    break;
                }
            }
        }

        state.restore_session_stream(sid, receiver).unwrap();
    };

    Ok(Sse::new(sse_stream))
}

pub async fn get_session_result(
    Path(sid): Path<SessionKey>,
    State(state): State<Arc<AppState>>
) -> impl IntoResponse {
    state.get_session_result(sid)
        .map(|result| (StatusCode::OK, Json(result)))
}

pub async fn abort_session(
    Path(sid): Path<SessionKey>,
    State(state): State<Arc<AppState>>
) -> impl IntoResponse {
    state.abort_session(sid)
        .map(|_| StatusCode::NO_CONTENT)
}

pub async fn destroy_session(
    Path(sid): Path<SessionKey>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    state.destroy_session(sid)
        .map(|_| StatusCode::NO_CONTENT)
}

pub async fn hibernate_session(
    Path(sid): Path<SessionKey>,
    State(state): State<Arc<AppState>>
) -> impl IntoResponse {
    state.hibernate_session(sid, &state.preference.sessions_directory).await
        .map(|_| StatusCode::NO_CONTENT)
}

pub async fn wakeup_session(
    Path(sid): Path<SessionKey>,
    State(state): State<Arc<AppState>>
) -> impl IntoResponse {
    state.wakeup_session(sid, &state.preference.sessions_directory).await
        .map(|_| StatusCode::NO_CONTENT)
}
