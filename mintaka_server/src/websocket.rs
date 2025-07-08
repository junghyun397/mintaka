use crate::app_state::AppState;
use crate::session::{SessionKey, SessionResponse};
use crate::stream_response_sender::StreamSessionResponseSender;
use axum::extract::ws;
use axum::extract::ws::WebSocket;
use futures_util::{SinkExt, StreamExt};
use mintaka::config::Config;
use mintaka::protocol::response::Response;
use rusty_renju::board::Board;
use rusty_renju::history::History;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamMap;
use tracing::info;

pub async fn handle_socket(
    mut socket: WebSocket,
    mut state: Arc<AppState>,
) {
    info!("new connection");

    let mut stream_map: StreamMap<SessionKey, UnboundedReceiverStream<SessionResponse>> = StreamMap::new();

    let (mut sender, mut receiver) = socket.split();

    loop {
        tokio::select! {
            Some(Ok(msg)) = receiver.next() => {
                match msg {
                    ws::Message::Text(text) => {
                        match text.as_str() {
                            "acquire" => {
                                let session_key = SessionKey::new_random();

                                if let Some(session_response_stream) = state.acquire_session_stream(session_key).unwrap() {
                                    stream_map.insert(session_key, session_response_stream);
                                }
                            }
                            "new" => {
                                let config = Config::default();
                                let board = Board::default();
                                let history = History::default();

                                let (session_key, session_response_stream) = state.new_session(config, board, history);

                                stream_map.insert(session_key, session_response_stream);
                            },
                            _ => {
                                sender.send(ws::Message::Text("invalid command".into())).await.unwrap();
                            }
                        }
                    },
                    ws::Message::Binary(data) => {
                    },
                    ws::Message::Close(_) => {
                        break;
                    }
                    _ => {}
                }
            },
            Some((session_key, session_response)) = stream_map.next() => {
                match session_response {
                    SessionResponse::Response(response) => {
                    },
                    SessionResponse::BestMove(best_move) => {
                    },
                    SessionResponse::Terminate => {
                        stream_map.remove(&session_key);
                    }
                }
            }
        }
    }

    for session_key in stream_map.keys().copied().collect::<Vec<_>>() {
        let session_response_stream = stream_map.remove(&session_key).unwrap();

        state.recover_session_stream(session_key, session_response_stream).unwrap();
    }

    info!("connection closed");
}
