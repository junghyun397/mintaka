mod preference;
mod session;
mod stream_response_sender;
mod app_state;

use crate::app_state::AppState;
use crate::preference::Preference;
use crate::session::SessionKey;
use axum::extract::ws::WebSocket;
use axum::extract::{ws, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use axum::ServiceExt;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashSet;
use std::env;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::log::Level::Trace;
use tracing::{info, info_span, Instrument};
use tracing_subscriber::EnvFilter;
use uuid::Timestamp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pref = Preference::from_args(env::args()).unwrap_or_default();

    tracing_subscriber::fmt()
        .init();

    let addr: SocketAddr = pref.address.parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    let state = Arc::new(AppState::new(pref));

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    info!("listening on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(
    mut socket: WebSocket,
    state: Arc<AppState>,
) {
    let mut interests: HashSet<SessionKey> = HashSet::new();

    let (mut sender, mut receiver) = socket.split();

    loop {
        tokio::select! {
            Some(Ok(msg)) = receiver.next() => {
                match msg {
                    ws::Message::Text(text) => {
                        sender.send(ws::Message::Text(text)).await.unwrap();
                    },
                    ws::Message::Binary(data) => {
                    },
                    ws::Message::Ping(_) => {},
                    ws::Message::Pong(_) => {},
                    ws::Message::Close(_) => {
                        break;
                    }
                }
            }
        }
    }
}
