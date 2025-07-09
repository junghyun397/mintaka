use crate::app_state::AppState;
use crate::session::SessionResponse;
use axum::extract::State;
use axum::response::Sse;
use axum::Json;
use mintaka::config::Config;
use rusty_renju::board::Board;
use rusty_renju::history::History;
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
pub struct Health {
    available_workers: usize,
}

pub async fn health(State(state): State<Arc<AppState>>) -> Json<Health> {
    Json(Health {
        available_workers: state.available_workers()
    })
}

pub async fn new_session(State(state): State<Arc<AppState>>) -> Json<Uuid> {
    let config = Config::default();
    let board = Board::default();
    let history = History::default();

    let session_key = state.new_session(config, board, history);

    Json(session_key.into())
}

pub async fn launch_session(State(state): State<Arc<AppState>>) {
    todo!()
}

pub async fn subscribe_session(State(state): State<Arc<AppState>>) -> Sse<SessionResponse> {
    todo!()
}
