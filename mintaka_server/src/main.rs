use axum::body::Body;
use axum::extract::{ConnectInfo, FromRequestParts, Path, State};
use axum::http::request::Parts;
use axum::http::{HeaderName, HeaderValue, Method, Request, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{middleware, RequestExt, Router};
use axum_server::tls_rustls::RustlsConfig;
use mintaka_server::app_error::AppError;
use mintaka_server::app_state::AppState;
use mintaka_server::preference::{Preference, TlsConfig};
use mintaka_server::rest;
use mintaka_server::session::{SessionData, SessionKey, SessionStatus};
use std::error::Error;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::signal::unix::SignalKind;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::{DefaultOnFailure, TraceLayer};
use tracing::{field, Span};

async fn auth(
    State(state): State<Arc<AppState>>,
    request: axum::extract::Request,
    next: Next
) -> impl IntoResponse {
    if request.method() == Method::OPTIONS {
        return Ok(next.run(request).await);
    }

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut pref = Preference::parse();

    tracing_subscriber::fmt()
        .with_target(false)
        .init();

    check_session_directory(&pref.sessions_directory);

    let addr: SocketAddr = pref.address.parse()?;

    let state = Arc::new(AppState::new(pref.clone()));

    let mut api = Router::new()
        .route("/status", get(rest::status))
        .route("/sessions", post(rest::new_session))
        .route("/sessions/{sid}", get(rest::check_session))
        .route("/sessions/{sid}", delete(rest::destroy_session))
        .route("/sessions/{sid}/configs", get(rest::get_session_configs))
        .route("/sessions/{sid}/commands", post(rest::command_session))
        .route("/sessions/{sid}/launch", post(rest::launch_session))
        .route("/sessions/{sid}/stream", get(rest::subscribe_session_response))
        .route("/sessions/{sid}/abort", post(rest::abort_session))
        .route("/sessions/{sid}/result", get(rest::get_session_result))
        .route("/sessions/{sid}/hibernate", post(rest::hibernate_session))
        .route("/sessions/{sid}/wakeup", post(rest::wakeup_session))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &axum::extract::Request| {
                    let span = tracing::info_span!(
                        "http",
                        ip = field::Empty,
                        method = %req.method(),
                        uri = %req.uri(),
                    );

                    if let Some(ConnectInfo(addr)) = req.extensions().get::<ConnectInfo<SocketAddr>>() {
                        span.record("ip", &field::display(addr));
                    }

                    span
                })
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
        .with_state(state.clone());

    if pref.api_password.is_some() {
        tracing::info!("password protected; use Api-Password header to authenticate");
    } else {
        tracing::warn!("API is not password protected; set MINTAKA_API_PASSWORD environment variable to enable protection");
    }

    state.wakeup_all_sessions().await?;

    spawn_session_cleaner(&state);
    spawn_hibernation_watcher(&state);
    spawn_hibernation_cleaner(&pref.sessions_directory);

    let url = if pref.tls_config.is_some() {
        format!("https://{}", pref.address)
    } else {
        format!("http://{}", pref.address)
    };

    if pref.webui {
        let webui = Router::new()
            .fallback_service(ServeDir::new("mintaka_webui/dist"))
            .layer(SetResponseHeaderLayer::overriding(
                HeaderName::from_static("cross-origin-opener-policy"),
                HeaderValue::from_static("same-origin"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                HeaderName::from_static("cross-origin-embedder-policy"),
                HeaderValue::from_static("require-corp"),
            ));

        api = api.merge(webui);

        tracing::info!("serving mintaka-webui on {url}");
    }

    if let Some(tls_config) = &pref.tls_config {
        let ruslts_config = RustlsConfig::from_pem_file(
            std::path::Path::new(&tls_config.cert_path),
            std::path::Path::new(&tls_config.key_path),
        ).await?;

        if tls_config.observe_sighup {
            spawn_tls_watcher(ruslts_config.clone(), tls_config.clone());
        }

        tracing::info!("api listening on {url}");

        open_browser(&pref, &url);

        axum_server::bind_rustls(addr, ruslts_config)
            .serve(api.into_make_service_with_connect_info::<SocketAddr>())
            .await?;
    } else {
        let listener = tokio::net::TcpListener::bind(addr)
            .await?;

        tracing::info!("api listening on {url}");

        open_browser(&pref, &url);

        axum::serve(listener, api.into_make_service_with_connect_info::<SocketAddr>())
            .await?;
    }

    Ok(())
}

fn open_browser(pref: &Preference, url: &str) {
    if pref.open_webui {
        let _ = webbrowser::open(&url);
    }
}

fn check_session_directory(session_directory: &str) {
    if !std::path::Path::new(session_directory).exists() {
        std::fs::create_dir_all(session_directory).unwrap();

        tracing::info!("created session directory: {}", session_directory);
    }
}

fn spawn_tls_watcher(shared_rustls_config: RustlsConfig, tls_config: TlsConfig) {
    tracing::info!("watching SIGHUP for renew TLS certs: {}, {}", tls_config.cert_path, tls_config.key_path);

    let alt_tls_config = tls_config.clone();

    tokio::spawn(async move {
        let mut signal_stream = tokio::signal::unix::signal(SignalKind::hangup()).unwrap();

        while signal_stream.recv().await.is_some() {
            tracing::info!("received SIGHUP signal; reload TLS certs");

            if let Err(e) = shared_rustls_config
                .reload_from_pem_file(&alt_tls_config.cert_path, &alt_tls_config.key_path)
                .await
            {
                tracing::error!("failed to reload TLS certs: {}", e);
            }

            tracing::info!("reloaded TLS certs")
        }
    });
}

fn spawn_hibernation_watcher(state: &Arc<AppState>) {
    tracing::info!("watching SIGUSR1 for hibernate all sessions and exit");

    let state = state.clone();

    tokio::spawn(async move {
        let mut signal_stream = tokio::signal::unix::signal(SignalKind::user_defined1()).unwrap();

        if let Some(_) = signal_stream.recv().await {
            tracing::info!("received SIGUSR1 signal; hibernate all sessions and exit");

            state.hibernate_all_sessions().await.unwrap();

            std::process::exit(0);
        }
    });
}

fn spawn_session_cleaner(state: &Arc<AppState>) {
    let state = state.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));

        loop {
            interval.tick().await;

            let now = Instant::now();

            let expired_keys: Vec<SessionKey> = state.sessions.map.iter()
                .filter(|session|
                    session.status() == SessionStatus::Idle && session.is_expired(now)
                )
                .map(|session| *session.key())
                .collect();

            for session_key in expired_keys {
                let _ = state.destroy_session(session_key);
            }
        }
    });
}

fn spawn_hibernation_cleaner(directory: &str) {
    let directory = directory.to_string();

    tracing::info!("watching hibernated sessions for expiry");

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));

        loop {
            interval.tick().await;

            let now_epoch_secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();


            let mut read_dir = tokio::fs::read_dir(&directory).await.unwrap();

            while let Some(entry) = read_dir.next_entry().await.unwrap() {
                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();

                let Some((sid_str, expiry_str)) = file_name.split_once('_') else {
                    continue;
                };

                let Ok(session_key) = SessionKey::from_str(sid_str) else {
                    continue;
                };

                let Ok(expiry_epoch_secs) = expiry_str.parse::<u64>() else {
                    continue;
                };

                if expiry_epoch_secs < now_epoch_secs {
                    continue;
                }

                let file_path = entry.path();

                if let Err(err) = tokio::fs::remove_file(&file_path).await {
                    tracing::warn!("failed to remove expired hibernated session: path={}, err={err}",file_path.display());

                    continue;
                }

                tracing::info!("removed expired hibernated session: sid={session_key}");
            }
        }
    });
}
