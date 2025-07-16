use crate::app_state::{AppError, AppState};
use crate::session::{SessionKey, SessionResponse};
use axum::extract::{ConnectInfo, Path, State};
use axum::http::StatusCode;
use axum::response::{sse, IntoResponse, Sse};
use axum::Json;
use futures_util::Stream;
use mintaka::config::Config;
use mintaka::game_agent::GameError;
use mintaka::protocol::command::Command;
use rusty_renju::board::Board;
use rusty_renju::history::History;
use serde::Serialize;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio_stream::StreamExt;
use tracing::info;

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
            AppError::GameError(game_error) => match game_error {
                GameError::StoneAlreadyExist => (StatusCode::CONFLICT, "stone already exist".to_string()),
                GameError::StoneDoesNotExist => (StatusCode::CONFLICT, "stone does not exist".to_string()),
                GameError::StoneColorMismatch => (StatusCode::CONFLICT, "stone color mismatch".to_string()),
                GameError::ForbiddenMove => (StatusCode::CONFLICT, "forbidden move".to_string()),
                GameError::NoHistoryToUndo => (StatusCode::CONFLICT, "no history to undo".to_string()),
            },
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let config = Config::default();
    let board = Board::default();
    let history = History::default();

    let session_key = state.new_session(config, board, history);

    info!("new session created: sid={session_key}, ip={}", addr.ip());

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
    info!("session launched: sid={sid}");

    state.launch_session(sid).await
        .map(|_| StatusCode::OK)
}

pub async fn subscribe_session_response(
    Path(sid): Path<SessionKey>,
    State(state): State<Arc<AppState>>
) -> anyhow::Result<Sse<impl Stream<Item = Result<sse::Event, Infallible>>>, AppError> {
    let session_stream = state.acquire_session_stream(sid)?;

    let sse_stream = session_stream
        .map(|session_response| {
            Ok(match session_response {
                SessionResponse::Response(response) => {
                    sse::Event::default()
                        .event("update")
                        .json_data(response).unwrap()
                },
                SessionResponse::BestMove(best_move) => {
                    sse::Event::default()
                        .event("best_move")
                        .json_data(best_move).unwrap()
                },
                SessionResponse::Terminate => {
                    sse::Event::default()
                        .event("terminate")
                }
            })
        });

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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(sid): Path<SessionKey>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!("session destroyed: sid={sid}, ip={}", addr.ip());

    state.destroy_session(sid)
        .map(|_| StatusCode::NO_CONTENT)
}

pub async fn hibernate_session(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(sid): Path<SessionKey>,
    State(state): State<Arc<AppState>>
) -> impl IntoResponse {
    info!("session hibernated: sid={sid}, ip={}", addr.ip());

    state.hibernate_session(sid).await
        .map(|_| StatusCode::OK)
}

pub async fn wakeup_session(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(sid): Path<SessionKey>,
    State(state): State<Arc<AppState>>
) -> impl IntoResponse {
    info!("session woken up: sid={sid}, ip={}", addr.ip());

    state.wakeup_session(sid).await
        .map(|_| StatusCode::OK)
}
