use crate::app_state::{AppError, AppState};
use crate::session::{SessionKey, SessionResponse};
use async_stream::stream;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{sse, IntoResponse, Sse};
use axum::Json;
use futures_util::Stream;
use mintaka::config::Config;
use mintaka::protocol::command::Command;
use mintaka::game_state::GameState;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use rusty_renju::hash_key::HashKey;
use rusty_renju::notation::rule::RuleKind;

impl From<&AppError> for StatusCode {
    fn from(error: &AppError) -> Self {
        match error {
            AppError::InvalidSessionId => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::InvalidConfig => StatusCode::INSUFFICIENT_STORAGE,
            AppError::SessionNotFound => StatusCode::NOT_FOUND,
            AppError::SessionInComputing => StatusCode::CONFLICT,
            AppError::SessionIdle => StatusCode::CONFLICT,
            AppError::SessionNeverLaunched => StatusCode::NO_CONTENT,
            AppError::SessionFileAlreadyExists => StatusCode::CONFLICT,
            AppError::SessionFileNotFound => StatusCode::NOT_FOUND,
            AppError::MemoryAcquireTimeout => StatusCode::SERVICE_UNAVAILABLE,
            AppError::WorkerAcquireTimeout => StatusCode::SERVICE_UNAVAILABLE,
            AppError::GameError(_) => StatusCode::CONFLICT,
            AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status_code = StatusCode::from(&self);
        let message = self.to_string();

        (status_code, message).into_response()
    }
}

#[derive(Serialize)]
#[typeshare::typeshare]
pub struct Health {
    version: String,
    available_workers: u32,
    available_memory_in_mib: u32,
}

pub async fn status(
    State(state): State<Arc<AppState>>
) -> impl IntoResponse {
    Json(Health {
        version: mintaka::VERSION.to_string(),
        available_workers: state.available_workers(),
        available_memory_in_mib: state.available_memory().mib() as u32,
    })
}

pub async fn check_session(
    Path(sid): Path<SessionKey>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    state.check_session(sid).await
        .map(|status| (StatusCode::OK, Json(status)))
}

#[derive(Deserialize)]
#[typeshare::typeshare]
pub struct CreateSessionRequest {
    api_password: Option<String>,
    config: Option<Config>,
    state: GameState<{ RuleKind::Renju }>,
}

#[derive(Serialize)]
#[typeshare::typeshare]
pub struct CreateSessionResponse {
    sid: String,
    token: String,
    hash: String,
    version: &'static str,
}

pub const SESSION_TOKEN_HEADER_NAME: &str = "mintaka-session-token";
pub const SESSION_TOKEN_QUERY_NAME: &str = "token";

pub async fn new_session(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateSessionRequest>
) -> Result<impl IntoResponse, AppError> {
    if let Some(expected_password) = &state.preference.api_password
        && payload.api_password.as_deref() != Some(expected_password.as_str())
    {
        return Err(AppError::Unauthorized);
    }

    let session = state.new_session(payload.config, payload.state).await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateSessionResponse {
            sid: session.key.to_string(),
            token: session.token.to_string(),
            hash: session.hash.to_string(),
            version: mintaka::VERSION,
        })
    ))
}

pub async fn get_session_configs(
    Path(sid): Path<SessionKey>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    state.configs_session(sid)
        .map(|configs| (StatusCode::OK, Json(configs)))
}

pub async fn command_session(
    Path(sid): Path<SessionKey>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Command>,
) -> impl IntoResponse {
    state.command_session(sid, payload)
        .map(|response| (StatusCode::ACCEPTED, Json(response)))
}

#[derive(Deserialize)]
#[typeshare::typeshare]
pub struct LaunchSessionRequest {
    position_hash: HashKey,
    nodes_polling_interval_in_ms: Option<u32>,
}

pub async fn launch_session(
    headers: HeaderMap,
    Path(sid): Path<SessionKey>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LaunchSessionRequest>
) -> impl IntoResponse {
    let timeout = headers.get("Timeout")
        .and_then(|timeout| timeout.to_str().ok())
        .and_then(|value| u64::from_str(value).ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(3));

    state.launch_session(sid, timeout, payload.position_hash, payload.nodes_polling_interval_in_ms)
        .await
        .map(|computing_resource| (StatusCode::OK, Json(computing_resource)))
}

pub async fn subscribe_session_response(
    Path(sid): Path<SessionKey>,
    State(state): State<Arc<AppState>>
) -> Result<Sse<impl Stream<Item = Result<sse::Event, Infallible>>>, AppError> {
    let mut receiver = state.subscribe_session_response(sid)?;

    let sse_stream = stream! {
        loop {
            match receiver.recv().await {
                Ok(SessionResponse::Response(response)) => {
                    yield Ok(sse::Event::default()
                        .event("Response")
                        .json_data(response)
                        .unwrap());
                },
                Ok(SessionResponse::BestMove(best_move)) => {
                    yield Ok(sse::Event::default()
                        .event("BestMove")
                        .json_data(best_move)
                        .unwrap());
                },
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {},
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    };

    Ok(Sse::new(sse_stream)
        .keep_alive(sse::KeepAlive::new().interval(Duration::from_secs(15))))
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
        .await
        .map(|_| StatusCode::NO_CONTENT)
}

pub async fn hibernate_session(
    Path(sid): Path<SessionKey>,
    State(state): State<Arc<AppState>>
) -> impl IntoResponse {
    state.hibernate_session(sid)
        .await
        .map(|_| StatusCode::NO_CONTENT)
}
