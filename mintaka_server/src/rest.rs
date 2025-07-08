use crate::app_state::AppState;
use axum::extract::State;
use axum::Json;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct Health {
    available_workers: usize,
}

pub async fn health(State(state): State<Arc<AppState>>) -> Json<Health> {
    Json(Health {
        available_workers: state.available_workers()
    })
}
