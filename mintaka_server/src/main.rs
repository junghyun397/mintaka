use crate::app_state::AppState;
use crate::preference::{Preference, TlsConfig};
use crate::session::SessionKey;
use axum::extract::{ConnectInfo, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, on_service, post};
use axum::{middleware, Router};
use axum_server::tls_rustls::RustlsConfig;
use std::error::Error;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::signal::unix::SignalKind;
use tower_http::trace::TraceLayer;
use tracing::log::warn;
use tracing::{error, info};

mod preference;
mod session;
mod stream_response_sender;
mod app_state;
mod rest;
mod websocket;
mod app_error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pref = Preference::parse();

    tracing_subscriber::fmt()
        .with_target(false)
        .init();

    check_session_directory(&pref.sessions_directory);

    let addr: SocketAddr = pref.address.parse()?;

    let mut state = AppState::new(pref.clone());

    state.spawn_session_cleanup();

    let shared_state = Arc::new(state);

    let app = Router::new()
        .route("/status", get(rest::status))
        .route("/sessions", post(rest::new_session))
        .route("/sessions/{sid}", delete(rest::destroy_session))
        .route("/sessions/{sid}/commands", post(rest::command_session))
        .route("/sessions/{sid}/launch", post(rest::launch_session))
        .route("/sessions/{sid}/stream", get(rest::subscribe_session_response))
        .route("/sessions/{sid}/abort", post(rest::abort_session))
        .route("/sessions/{sid}/result", get(rest::get_session_result))
        .route("/sessions/{sid}/hibernate", post(rest::hibernate_session))
        .route("/sessions/{sid}/wakeup", post(rest::wakeup_session))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &axum::extract::Request| {
                    if let Some(ConnectInfo(socket_addr)) = req
                        .extensions()
                        .get::<ConnectInfo<SocketAddr>>()
                    {
                        tracing::info_span!("http", ip = &socket_addr.ip().to_string())
                    } else {
                        tracing::info_span!("http unknown")
                    }
                })
        )
        .layer(middleware::from_fn_with_state(shared_state.clone(), auth))
        .with_state(shared_state.clone());

    if pref.api_password.is_some() {
        info!("password protected; use Api-Password header to authenticate");
    } else {
        warn!("API is not password protected; set MINTAKA_API_PASSWORD environment variable to enable protection");
    }

    spawn_hibernation_watcher(shared_state);

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
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await?;
    } else {
        let listener = tokio::net::TcpListener::bind(addr)
            .await?;

        info!("listening on http://{addr}, ws://{addr}/ws");

        axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
            .await?;
    }

    Ok(())
}

fn check_session_directory(session_directory: &str) {
    if !Path::new(session_directory).exists() {
        std::fs::create_dir_all(session_directory).unwrap();

        info!("created session directory: {}", session_directory);
    }
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

fn spawn_hibernation_watcher(shared_state: Arc<AppState>) {
    info!("watching SIGUSR1 for hibernate all sessions and exit");

    tokio::spawn(async move {
        let mut signal_stream = tokio::signal::unix::signal(SignalKind::user_defined1()).unwrap();

        if let Some(_) = signal_stream.recv().await {
            info!("received SIGUSR1 signal; hibernate all sessions and exit");

            shared_state.hibernate_all_sessions().await.unwrap();

            std::process::exit(0);
        }
    });
}

async fn auth(
    State(state): State<Arc<AppState>>,
    request: axum::extract::Request,
    next: middleware::Next
) -> impl IntoResponse {
    if let Some(expected_password) = &state.preference.api_password {
        let password = request.headers()
            .get("Api-Password")
            .and_then(|h| h.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        if password != expected_password {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    Ok(next.run(request).await)
}
