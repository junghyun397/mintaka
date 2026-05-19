use axum::extract::{ConnectInfo, Path, Request, State};
use axum::http::{HeaderName, HeaderValue, Method, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{middleware, Router};
use axum_server::tls_rustls::RustlsConfig;
use mintaka_server::app_state::AppState;
use mintaka_server::preference::{Preference, TlsConfig};
use mintaka_server::rest;
use mintaka_server::session::{SessionKey, SessionStatus, SessionToken};
use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::signal::unix::SignalKind;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use tracing::field;

async fn auth_with_header(
    State(state): State<Arc<AppState>>,
    Path(session_key): Path<String>,
    request: Request,
    next: Next
) -> Result<impl IntoResponse, StatusCode> {
    auth_session(state, session_key, session_token_from_header(&request), request, next).await
}

async fn auth_with_query(
    State(state): State<Arc<AppState>>,
    Path(session_key): Path<String>,
    request: Request,
    next: Next
) -> Result<impl IntoResponse, StatusCode> {
    auth_session(state, session_key, session_token_from_query(&request), request, next).await
}

async fn auth_session(
    state: Arc<AppState>,
    session_key: String,
    session_token: Option<SessionToken>,
    request: Request,
    next: Next
) -> Result<impl IntoResponse, StatusCode> {
    if request.method() != Method::OPTIONS {
        let session_key = SessionKey::from_str(&session_key)
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        let session_token = session_token
            .ok_or(StatusCode::NOT_FOUND)?;

        state.authorize_session(session_key, session_token)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;
    }

    Ok(next.run(request).await)
}

fn session_token_from_header(request: &Request) -> Option<SessionToken> {
    request.headers()
        .get(rest::SESSION_TOKEN_HEADER_NAME)
        .and_then(|h| h.to_str().ok())
        .and_then(|value| SessionToken::from_str(value).ok())
}

fn session_token_from_query(request: &Request) -> Option<SessionToken> {
    request.uri().query()?
        .split('&')
        .filter_map(|param| param.split_once('='))
        .find_map(|(name, value)| {
            (name == rest::SESSION_TOKEN_QUERY_NAME)
                .then(|| SessionToken::from_str(value).ok())?
        })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pref = Preference::parse();

    tracing_subscriber::fmt()
        .with_target(false)
        .init();

    check_session_directory(&pref.sessions_directory);

    let addr: SocketAddr = pref.address.parse()?;

    let state = Arc::new(AppState::new(pref.clone())?);

    let session_routes = Router::new()
        .route("/{sid}", get(rest::check_session).delete(rest::destroy_session))
        .route("/{sid}/configs", get(rest::get_session_configs))
        .route("/{sid}/commands", post(rest::command_session))
        .route("/{sid}/launch", post(rest::launch_session))
        .route("/{sid}/abort", post(rest::abort_session))
        .route("/{sid}/result", get(rest::get_session_result))
        .route("/{sid}/hibernate", post(rest::hibernate_session))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_with_header));

    let session_stream_routes = Router::new()
        .route("/{sid}/stream", get(rest::subscribe_session_response))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_with_query));

    let mut api = Router::new()
        .route("/status", get(rest::status))
        .route("/sessions", post(rest::new_session))
        .nest("/sessions", session_routes.merge(session_stream_routes))
        .layer(CorsLayer::very_permissive())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &Request| {
                    let span = tracing::info_span!(
                        "http",
                        ip = field::Empty,
                        method = %req.method(),
                        uri = %req.uri().path(),
                    );

                    if let Some(ConnectInfo(addr)) = req.extensions().get::<ConnectInfo<SocketAddr>>() {
                        span.record("ip", &field::display(addr));
                    }

                    span
                })
        )
        .with_state(state.clone());

    if pref.api_password.is_some() {
        tracing::info!("password protected; use api_password in POST /sessions body to create a session");
    } else {
        tracing::warn!("session creation is not password protected; set MINTAKA_API_PASSWORD environment variable to enable protection");
    }

    spawn_session_cleaner(&state);
    spawn_sigterm_watcher(&state);
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

fn spawn_sigterm_watcher(state: &Arc<AppState>) {
    tracing::info!("watching SIGTERM for hibernate all sessions and exit");

    let state = state.clone();

    tokio::spawn(async move {
        let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate()).unwrap();

        sigterm.recv().await;
        tracing::info!("received SIGTERM signal; hibernate all sessions and exit");

        if let Err(err) = state.hibernate_all_sessions().await {
            tracing::warn!("failed to hibernate all sessions: {err}");
        }

        std::process::exit(0);
    });
}

fn spawn_session_cleaner(state: &Arc<AppState>) {
    let state = state.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));

        loop {
            interval.tick().await;

            let now = Instant::now();
            let hibernate_timeout = state.preference.hibernate_timeout;

            let mut hibernation_keys = vec![];
            let mut expired_keys = vec![];

            for session in state.sessions.map.iter()
                .filter(|session| session.status() == SessionStatus::Idle)
            {
                if session.is_expired(now) {
                    expired_keys.push(*session.key());
                } else if hibernate_timeout
                    .is_some_and(|timeout| now.duration_since(session.last_active) > timeout)
                {
                    hibernation_keys.push(*session.key());
                }
            }

            for session_key in hibernation_keys {
                let _ = state.hibernate_session(session_key).await;
            }

            for session_key in expired_keys {
                let _ = state.destroy_session(session_key).await;
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

                let expiry_epoch_secs = match expiry_str {
                    "none" => None,
                    value => Some(match value.parse::<u64>() {
                        Ok(expiry) => expiry,
                        Err(_) => continue,
                    }),
                };

                if expiry_epoch_secs.is_none_or(|expiry| expiry >= now_epoch_secs) {
                    continue;
                }

                let file_path = entry.path();

                let result = tokio::fs::remove_file(&file_path).await;

                if let Err(err) = result {
                    tracing::warn!("failed to remove expired hibernated session: path={}, err={err}",file_path.display());

                    continue;
                }

                tracing::info!("removed expired hibernated session: sid={session_key}");
            }
        }
    });
}
