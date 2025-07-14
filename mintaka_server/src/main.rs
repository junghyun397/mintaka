use crate::app_state::AppState;
use crate::preference::{Preference, TlsConfig};
use axum::routing::{delete, get, post};
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::signal::unix::SignalKind;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

mod preference;
mod session;
mod stream_response_sender;
mod app_state;
mod rest;
mod websocket;
mod app_error;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pref = Preference::parse()?;

    tracing_subscriber::fmt()
        .init();

    let addr: SocketAddr = pref.address.parse()?;

    let state = Arc::new(AppState::new(pref.clone()));

    let app = Router::new()
        .route("/ws", get(websocket::websocket_handler))
        .route("/status", get(rest::status))
        .route("/sessions", post(rest::new_session))
        .route("/sessions/{sid}", delete(rest::destroy_session))
        .route("/sessions/{sid}/commands", post(rest::command_session))
        .route("/sessions/{sid}/launch", post(rest::launch_session))
        .route("/sessions/{sid}/stream", get(rest::subscribe_session_response))
        .route("/sessions/{sid}/abort", post(rest::abort_session))
        .route("/sessions/{sid}/result", get(rest::get_session_result))
        .route("/sessions/{sid}/hibernate", post(rest::hibernate_session))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    if let Some(tls_config) = pref.tls_config {
        let ruslts_config = RustlsConfig::from_pem_file(
            Path::new(&tls_config.cert_path),
            Path::new(&tls_config.key_path),
        ).await?;

        if tls_config.observe_sighup {
            spawn_tls_watcher(ruslts_config.clone(), tls_config.clone());
        }

        info!("listening on https://{addr}, wss://{addr}/ws");

        axum_server::bind_rustls(addr, ruslts_config)
            .serve(app.into_make_service())
            .await?;
    } else {
        let listener = tokio::net::TcpListener::bind(addr)
            .await?;

        info!("listening on http://{addr}, ws://{addr}/ws");

        axum::serve(listener, app)
            .await?;
    }

    Ok(())
}

fn spawn_tls_watcher(shared_rustls_config: RustlsConfig, tls_config: TlsConfig) {
    info!("watching SIGHUP for renew TLS certs: {}, {}", tls_config.cert_path, tls_config.key_path);

    let alt_tls_config = tls_config.clone();

    tokio::spawn(async move {
        let mut signal_stream = tokio::signal::unix::signal(SignalKind::hangup()).unwrap();

        while signal_stream.recv().await.is_some() {
            info!("received SIGHUP signal; reload TLS certs");

            if let Err(e) = shared_rustls_config
                .reload_from_pem_file(&alt_tls_config.cert_path, &alt_tls_config.key_path)
                .await
            {
                error!("failed to reload TLS certs: {}", e);
            }

            info!("reloaded TLS certs")
        }
    });
}
