use crate::app_state::{AppError, AppState};
use crate::session::{SessionKey, SessionResponse};
use async_stream::stream;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{sse, IntoResponse, Sse};
use axum::Json;
use futures_util::Stream;
use mintaka::config::Config;
use mintaka::state::GameState;
use mintaka::protocol::command::Command;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::StreamExt;

#[derive(Serialize)]
pub struct Health {
    available_workers: usize,
}

impl From<&AppError> for StatusCode {
    fn from(error: &AppError) -> Self {
        match error {
            AppError::InvalidSessionId => StatusCode::BAD_REQUEST,
            AppError::InvalidConfig => StatusCode::INSUFFICIENT_STORAGE,
            AppError::SessionNotFound => StatusCode::NOT_FOUND,
            AppError::SessionInComputing => StatusCode::CONFLICT,
            AppError::SessionIdle => StatusCode::CONFLICT,
            AppError::StreamAcquired => StatusCode::CONFLICT,
            AppError::StreamNotAcquired => StatusCode::CONFLICT,
            AppError::SessionNeverLaunched => StatusCode::NO_CONTENT,
            AppError::SessionFileAlreadyExists => StatusCode::CONFLICT,
            AppError::SessionFileNotFound => StatusCode::NOT_FOUND,
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

pub async fn status(
    State(state): State<Arc<AppState>>
) -> impl IntoResponse {
    Json(Health {
        available_workers: state.available_workers(),
    })
}

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    config: Config,
    state: GameState,
}

pub async fn check_session(
    Path(sid): Path<SessionKey>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    state.check_session(sid).await
        .map(|status| (StatusCode::OK, Json(status)))
}

pub async fn new_session(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateSessionRequest>
) -> impl IntoResponse {
    state.new_session(payload.config, payload.state)
        .await
        .map(|session_key| (StatusCode::CREATED, Json(session_key)))
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
    state.launch_session(sid)
        .await
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
                        .event("Response")
                        .json_data(response)
                        .unwrap());
                },
                SessionResponse::BestMove(best_move) => {
                    yield Ok(sse::Event::default()
                        .event("BestMove")
                        .json_data(best_move)
                        .unwrap());
                },
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
    state.hibernate_session(sid)
        .await
        .map(|_| StatusCode::NO_CONTENT)
}

pub async fn wakeup_session(
    Path(sid): Path<SessionKey>,
    State(state): State<Arc<AppState>>
) -> impl IntoResponse {
    state.wakeup_session(sid)
        .await
        .map(|_| StatusCode::NO_CONTENT)
}
