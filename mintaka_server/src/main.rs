mod preference;
mod session;
mod stream_response_sender;
mod app_state;

use crate::preference::Preference;
use axum::extract::ws::WebSocket;
use axum::extract::{ws, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use mintaka::protocol::response::Response;
use std::env;
use std::str::FromStr;
use time::OffsetDateTime;
use tokio_stream::StreamExt;

fn log_prefix() -> String {
    OffsetDateTime::now_utc().to_string()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pref = Preference::from_args(env::args()).unwrap_or_default();

    let app = Router::new()
        .route("/", get(websocket_handler));

    Ok(())
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(
    mut socket: WebSocket
) {
    let (mut sender, mut receiver) = socket.split();

    let mut response_stream = tokio_stream::empty::<Response>();

    loop {
        tokio::select! {
            Some(Ok(msg)) = receiver.next() => {
                match msg {
                    ws::Message::Text(text) => {
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
            Some(response) = response_stream.next() => {
                match response {
                    Response::Begins{ .. } => {
                    },
                    Response::Status{ .. } => {
                    },
                    _ => {}
                }
            }
        }
    }
}
