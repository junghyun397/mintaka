pub(crate) use crate::app_error::AppError;
use crate::preference::Preference;
use crate::session::{Session, SessionData, SessionKey, SessionResponse, SessionResponseReceiver, SessionResponseSender, SessionResultResponse, SessionStatus, SessionToken, Sessions};
use crate::stream_response_sender::StreamSessionResponseSender;
use mintaka::config::Config;
use mintaka::protocol::command::Command;
use mintaka::game_state::GameState;
use rusty_renju::utils::byte_size::ByteSize;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};
use mintaka::game_agent::{GameError};
use mintaka::protocol::results::{BestMove, CommandResult};
use rusty_renju::hash_key::HashKey;
use rusty_renju::notation::rule::RuleKind;
use crate::app_error::AppError::SessionInComputing;

const SESSION_RESPONSE_CHANNEL_CAPACITY: usize = 4;
const RESOURCE_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(3);

struct HibernatedSessionFile {
    path: PathBuf,
    meta_path: PathBuf,
    token: SessionToken,
}

#[derive(Serialize, Deserialize)]
struct HibernatedSessionMeta {
    token: SessionToken,
    expiry_epoch_secs: Option<u64>,
}

fn hibernated_session_file_name(session_key: SessionKey, expiry_epoch_secs: &str) -> String {
    format!("{session_key}_{expiry_epoch_secs}")
}

fn hibernated_session_expiry_file_part(expiry_epoch_secs: Option<u64>) -> String {
    expiry_epoch_secs
        .map_or_else(|| "none".to_string(), |expiry| expiry.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Configs {
    pub default_config: Config,
    pub max_config: Config,
    pub config: Config,
}

pub struct CreatedSession {
    pub key: SessionKey,
    pub token: SessionToken,
    pub hash: HashKey,
}

pub struct WorkerPermit(OwnedSemaphorePermit);

impl WorkerPermit {
    pub fn release(self) {
        drop(self.0);
    }
}

pub struct MemoryPermit(OwnedSemaphorePermit);

impl MemoryPermit {
    pub fn release(self) {
        drop(self.0);
    }
}

#[derive(Debug)]
pub struct SessionResource {
    pub memory: ByteSize,
    pub workers: u32
}

impl From<&Config> for SessionResource {
    fn from(config: &Config) -> Self {
        Self {
            memory: config.tt_size,
            workers: config.workers,
        }
    }
}

impl SessionResource {
    const ZERO: SessionResource = SessionResource {
        memory: ByteSize::ZERO,
        workers: 0,
    };
}

pub struct AppState {
    pub sessions: Arc<Sessions>,
    memory_acquire_lock: Arc<Mutex<()>>,
    worker_resource: Arc<Semaphore>,
    memory_resource: Arc<Semaphore>,
    pub preference: Preference,
}

fn new_session_response_sender() -> SessionResponseSender {
    let (tx, _) = tokio::sync::broadcast::channel(SESSION_RESPONSE_CHANNEL_CAPACITY);
    tx
}

impl AppState {
    pub fn new(preference: Preference) -> Result<Self, AppError> {
        Ok(Self {
            sessions: Arc::new(Sessions::default()),
            memory_acquire_lock: Arc::new(Mutex::new(())),
            worker_resource: Arc::new(Semaphore::new(preference.cores)),
            memory_resource: Arc::new(Semaphore::new(preference.memory_limit.mib() as usize)),
            preference,
        })
    }

    pub fn max_config(&self) -> Option<Config> {
        self.preference.max_config
    }

    pub fn available_workers(&self) -> u32 {
        self.worker_resource.available_permits() as u32
    }

    pub async fn acquire_workers(&self, workers: u32, timeout: Duration) -> Result<WorkerPermit, AppError> {
        let worker_permit = tokio::time::timeout(
            timeout,
            self.worker_resource.clone().acquire_many_owned(workers)
        )
            .await
            .map_err(|_| AppError::WorkerAcquireTimeout)?
            .map_err(AppError::from_general_error)?;

        Ok(WorkerPermit(worker_permit))
    }

    pub fn available_memory(&self) -> ByteSize {
        ByteSize::from_mib(self.memory_resource.available_permits() as u64)
    }

    pub async fn acquire_memory(&self, memory_size: ByteSize, force_acquire: bool, timeout: Duration) -> Result<MemoryPermit, AppError> {
        let available = ByteSize::from_mib(self.memory_resource.available_permits() as u64);

        if memory_size > available && force_acquire {
            for (session_key, session) in self.sessions.remove_lru_idle_until_memory(memory_size) {
                if let Err(err) = self.hibernate_session(session_key, session).await {
                    tracing::warn!(
                        "failed to hibernate session for memory acquire: sid={session_key}, err={err}; skipping"
                    );
                }
            }
        }

        let memory_permit = tokio::time::timeout(
            timeout,
            self.memory_resource.clone().acquire_many_owned(memory_size.mib() as u32)
        )
            .await
            .map_err(|_| AppError::MemoryAcquireTimeout)?
            .map_err(AppError::from_general_error)?;

        Ok(MemoryPermit(memory_permit))
    }

    pub async fn check_session(&self, session_key: SessionKey) -> Result<SessionStatus, AppError> {
        match self.sessions.with(&session_key, |session| session.status()) {
            Some(status) => Ok(status),
            None => self.check_hibernated_session(session_key)
                .await
                .then_some(SessionStatus::Hibernating)
                .ok_or(AppError::SessionNotFound),
        }
    }

    pub async fn new_session(
        &self,
        config: Option<Config>,
        game_state: GameState<{ RuleKind::Renju }>,
        time_to_hibernate: Option<Duration>,
        time_to_live: Option<Duration>,
    ) -> Result<CreatedSession, AppError> {
        if let Some(config) = config
            && let Some(max_config) = self.preference.max_config
            && config > max_config
        {
            return Err(AppError::InvalidConfig);
        }

        let config = config.unwrap_or(self.preference.default_config);

        let session_key = SessionKey::new_random();
        let session_token = SessionToken::new_random();

        let _memory_acquire_guard = self.memory_acquire_lock.lock().await;
        let memory_permit = self.acquire_memory(config.tt_size, true, RESOURCE_ACQUIRE_TIMEOUT)
            .await?;

        let response_sender = new_session_response_sender();

        let hash_key = game_state.board.hash_key;

        let session = Session::new(
            config,
            session_token,
            game_state,
            time_to_hibernate, time_to_live,
            memory_permit,
            response_sender,
        );

        self.sessions.insert(session_key, session);

        tracing::info!("session created; sid={session_key}");

        Ok(CreatedSession {
            key: session_key,
            token: session_token,
            hash: hash_key,
        })
    }

    pub async fn authorize_session(&self, session_key: SessionKey, session_token: SessionToken, wakeup: bool) -> Result<(), AppError> {
        let active_session_token = self.sessions.with(&session_key, |session| session.token);

        let expected_token = match active_session_token {
            Some(token) => token,
            None => self.hibernated_session_file(session_key)
                .await?
                .ok_or(AppError::SessionNotFound)?
                .token,
        };

        if expected_token != session_token {
            return Err(AppError::Unauthorized);
        }

        if active_session_token.is_none() && wakeup {
            let _ = self.wakeup_session(session_key).await?;
        }

        Ok(())
    }

    pub fn configs_session(&self, session_key: SessionKey) -> Result<Configs, AppError> {
        let config = self.sessions.with(&session_key, |session| session.config)
            .ok_or(AppError::SessionNotFound)?;

        Ok(Configs {
            default_config: self.preference.default_config,
            max_config: self.preference.max_config.unwrap_or(Config::UNLIMITED_CONFIG),
            config,
        })
    }

    pub async fn hibernate_active_session(&self, session_key: SessionKey) -> Result<(), AppError> {
        let session = self.sessions.remove_idle(&session_key, None)?;

        self.hibernate_session(session_key, session).await
    }

    async fn hibernate_session(&self, session_key: SessionKey, session: Session) -> Result<(), AppError> {
        if session.status() != SessionStatus::Idle {
            return Err(SessionInComputing)
        }

        let encoded = rmp_serde::to_vec(&session)
            .map_err(AppError::from_general_error)?;
        let meta = HibernatedSessionMeta {
            token: session.token,
            expiry_epoch_secs: session.live_until_epoch_secs(),
        };

        let expiry_epoch_secs = hibernated_session_expiry_file_part(meta.expiry_epoch_secs);
        let file_name = hibernated_session_file_name(session_key, &expiry_epoch_secs);
        let file_path = self.hibernated_session_file_path(&file_name);
        let tmp_file_path = file_path.with_extension("tmp");
        let meta_path = self.hibernated_session_meta_file_path(session_key);
        let tmp_meta_path = meta_path.with_extension("meta.tmp");

        let _ = self.remove_hibernated_session_file(session_key).await;

        let mut file = tokio::fs::File::create(&tmp_file_path)
            .await
            .map_err(|_| AppError::SessionFileAlreadyExists)?;

        if let Err(err) = file.write_all(&encoded).await {
            let _ = tokio::fs::remove_file(&tmp_file_path).await;

            return Err(AppError::from_general_error(err));
        }

        if let Err(err) = file.flush().await {
            let _ = tokio::fs::remove_file(&tmp_file_path).await;

            return Err(AppError::from_general_error(err));
        }

        drop(file);

        if let Err(err) = tokio::fs::rename(&tmp_file_path, &file_path).await {
            let _ = tokio::fs::remove_file(&tmp_file_path).await;

            return Err(AppError::from_general_error(err));
        }

        let encoded_meta = rmp_serde::to_vec(&meta)
            .map_err(AppError::from_general_error)?;

        if let Err(err) = tokio::fs::write(&tmp_meta_path, encoded_meta).await {
            let _ = tokio::fs::remove_file(&tmp_meta_path).await;
            let _ = tokio::fs::remove_file(&file_path).await;

            return Err(AppError::from_general_error(err));
        }

        if let Err(err) = tokio::fs::rename(&tmp_meta_path, &meta_path).await {
            let _ = tokio::fs::remove_file(&tmp_meta_path).await;
            let _ = tokio::fs::remove_file(&file_path).await;

            return Err(AppError::from_general_error(err));
        }

        tracing::info!("session hibernated");

        Ok(())
    }

    pub async fn check_hibernated_session(&self, session_key: SessionKey) -> bool {
        self.hibernated_session_path(session_key)
            .await
            .ok()
            .flatten()
            .is_some()
    }

    pub async fn wakeup_session(&self, session_key: SessionKey) -> Result<(), AppError> {
        if self.sessions.with(&session_key, |_| ()).is_some() {
            let _ = self.remove_hibernated_session_file(session_key).await;

            return Ok(());
        }

        let (file, session_data) = self.load_hibernated_session(session_key).await?;
        let session_token = file.token;

        let _memory_acquire_guard = self.memory_acquire_lock.lock().await;
        let memory_permit = self.acquire_memory(session_data.config.tt_size, true, RESOURCE_ACQUIRE_TIMEOUT)
            .await?;

        let response_sender = new_session_response_sender();

        let session = tokio::task::spawn_blocking(move ||
            Session::from_data(session_data, session_token, memory_permit, response_sender)
        )
            .await
            .map_err(AppError::from_general_error)?
            .map_err(AppError::from_general_error)?;

        if self.sessions.try_insert(session_key, session).is_err() {
            let _ = self.remove_hibernated_session_file(session_key).await;

            return Ok(());
        }

        self.remove_hibernated_session_file(session_key).await?;

        tracing::info!("session woken up");

        Ok(())
    }

    async fn load_hibernated_session(&self, session_key: SessionKey) -> Result<(HibernatedSessionFile, SessionData), AppError> {
        let file = self.hibernated_session_file(session_key)
            .await?
            .ok_or(AppError::SessionFileNotFound)?;

        let buf = tokio::fs::read(&file.path)
            .await
            .map_err(|_| AppError::SessionFileNotFound)?;

        let session_data = rmp_serde::from_slice(&buf)
            .map_err(AppError::from_general_error)?;

        Ok((file, session_data))
    }

    fn hibernated_session_meta_file_path(&self, session_key: SessionKey) -> PathBuf {
        Path::new(&self.preference.sessions_directory).join(format!("{session_key}.meta"))
    }

    fn hibernated_session_file_path(&self, file_name: &str) -> PathBuf {
        Path::new(&self.preference.sessions_directory).join(file_name)
    }

    async fn hibernated_session_path(&self, session_key: SessionKey) -> Result<Option<PathBuf>, AppError> {
        Ok(self.hibernated_session_file(session_key)
            .await?
            .map(|file| file.path))
    }

    async fn hibernated_session_file(&self, session_key: SessionKey) -> Result<Option<HibernatedSessionFile>, AppError> {
        let now_epoch_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH).unwrap()
            .as_secs();

        let meta_path = self.hibernated_session_meta_file_path(session_key);
        let meta_buf = match tokio::fs::read(&meta_path).await {
            Ok(buf) => buf,
            Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(AppError::from_general_error(err)),
        };

        let meta: HibernatedSessionMeta = rmp_serde::from_slice(&meta_buf)
            .map_err(AppError::from_general_error)?;

        let expiry_file_part = hibernated_session_expiry_file_part(meta.expiry_epoch_secs);
        let file_name = hibernated_session_file_name(session_key, &expiry_file_part);
        let path = self.hibernated_session_file_path(&file_name);

        if meta.expiry_epoch_secs.is_some_and(|expiry| expiry < now_epoch_secs) {
            let _ = tokio::fs::remove_file(&path).await;
            let _ = tokio::fs::remove_file(&meta_path).await;

            return Ok(None);
        }

        match tokio::fs::metadata(&path).await {
            Ok(_) => {}
            Err(err) if err.kind() == ErrorKind::NotFound => {
                let _ = tokio::fs::remove_file(&meta_path).await;

                return Ok(None);
            }
            Err(err) => return Err(AppError::from_general_error(err)),
        }

        Ok(Some(HibernatedSessionFile {
            path,
            meta_path,
            token: meta.token,
        }))
    }

    async fn remove_hibernated_session_file(&self, session_key: SessionKey) -> Result<bool, AppError> {
        let Some(file) = self.hibernated_session_file(session_key).await? else {
            return Ok(false);
        };

        let mut removed = false;

        for path in [&file.path, &file.meta_path] {
            match tokio::fs::remove_file(path).await {
                Ok(()) => removed = true,
                Err(err) if err.kind() == ErrorKind::NotFound => {}
                Err(err) => return Err(AppError::from_general_error(err)),
            }
        }

        Ok(removed)
    }

    pub async fn clean_idle_sessions(&self) {
        enum CleanupAction {
            Hibernate,
            Destroy,
        }

        let now = Instant::now();

        for session_key in self.sessions.keys() {
            let action = self.sessions.with(&session_key, |session| {
                if session.status() != SessionStatus::Idle {
                    None
                } else if session.is_expired(now) {
                    Some(CleanupAction::Destroy)
                } else if session.should_hibernate(now) {
                    Some(CleanupAction::Hibernate)
                } else {
                    None
                }
            }).flatten();

            match action {
                Some(CleanupAction::Hibernate) => {
                    if let Err(err) = self.hibernate_active_session(session_key).await {
                        tracing::warn!("failed to hibernate idle session: sid={session_key}, err={err}; skipping");
                    }
                }
                Some(CleanupAction::Destroy) => {
                    if let Err(err) = self.destroy_active_session(session_key) {
                        tracing::warn!("failed to destroy expired session: sid={session_key}, err={err}; skipping");
                    }
                }
                None => {}
            }
        }

        self.sessions.compact_eviction_entries();
    }

    pub async fn clean_expired_hibernated_sessions(&self) -> Result<(), AppError> {
        let mut entries = tokio::fs::read_dir(&self.preference.sessions_directory)
            .await
            .map_err(AppError::from_general_error)?;

        let now_epoch_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH).unwrap()
            .as_secs();

        while let Some(entry) = entries.next_entry().await.map_err(AppError::from_general_error)? {
            let file_name = entry.file_name();

            let Some(file_name) = file_name.to_str() else {
                continue;
            };

            let Some((session_key, expiry_epoch_secs)) = file_name.split_once('_') else {
                continue;
            };

            let Ok(session_key) = session_key.parse() else {
                continue;
            };

            let Ok(expiry_epoch_secs) = expiry_epoch_secs.parse::<u64>() else {
                continue;
            };

            if expiry_epoch_secs >= now_epoch_secs {
                continue;
            }

            self.remove_hibernated_session_file(session_key).await?;
        }

        Ok(())
    }

    pub fn command_session(
        &self,
        session_key: SessionKey,
        command: Command,
    ) -> Result<CommandResult, AppError> {
        self.sessions.with_mut_touch(&session_key, |session| session.command(command))
            .ok_or(AppError::SessionNotFound)?
    }

    pub async fn launch_session(
        &self,
        session_key: SessionKey,
        timeout: Duration,
        position_hash: HashKey,
        nodes_polling_interval_ms: Option<u32>,
    ) -> Result<(), AppError> {
        let workers = self.sessions.with_touch(&session_key, |session| {
            if session.status() != SessionStatus::Idle {
                return Err(SessionInComputing)
            }

            if position_hash != session.game_agent()?.state.board.hash_key {
                return Err(AppError::GameError(GameError::HashMismatch))
            }

            Ok(session.config.workers)
        })
            .ok_or(AppError::SessionNotFound)??;

        let worker_permit = self.acquire_workers(workers, timeout).await?;

        let (result_tx, result_rx) = tokio::sync::oneshot::channel();

        let response_sender = self.sessions.with_mut_touch(&session_key, move |session| {
            if session.status() != SessionStatus::Idle {
                return Err(SessionInComputing)
            }

            if position_hash != session.game_agent()?.state.board.hash_key {
                return Err(AppError::GameError(GameError::HashMismatch))
            }

            let response_sender = session.response_sender.clone();

            session.launch(
                StreamSessionResponseSender::new(response_sender.clone()),
                result_tx,
                worker_permit,
                nodes_polling_interval_ms,
            )?;

            Ok(response_sender)
        })
            .ok_or(AppError::SessionNotFound)??;

        let sessions = self.sessions.clone();

        tokio::spawn(async move {
            match result_rx.await {
                Ok(SessionResultResponse { game_agent, best_move, .. }) => {
                    let _ = sessions.with_mut_touch(&session_key, |session| {
                        session.store_best_move(best_move);
                        session.restore(game_agent).unwrap();
                    });

                    let _ = response_sender.send(SessionResponse::BestMove(best_move));
                }
                Err(err) => {
                    eprintln!("{err}")
                }
            }
        });

        tracing::info!("session launched");

        Ok(())
    }

    pub fn abort_session(&self, session_key: SessionKey) -> Result<(), AppError> {
        self.sessions.with_touch(&session_key, |session| {
            if session.status() == SessionStatus::Idle {
                return Err(AppError::SessionIdle);
            }

            session.abort()
        })
            .ok_or(AppError::SessionNotFound)??;

        tracing::info!("session aborted");

        Ok(())
    }

    pub fn get_session_result(&self, session_key: SessionKey) -> Result<BestMove, AppError> {
        self.sessions.with(&session_key, |session| session.last_best_move())
            .ok_or(AppError::SessionNotFound)?
            .ok_or(AppError::SessionNeverLaunched)
    }

    fn destroy_active_session(&self, session_key: SessionKey) -> Result<SessionResource, AppError> {
        let session = self.sessions.remove_idle(&session_key, None)?;

        let resource = (&session.config).into();

        tracing::info!("session destroyed; resource={resource:?}");

        Ok(resource)
    }

    pub async fn destroy_session(&self, session_key: SessionKey) -> Result<SessionResource, AppError> {
        match self.destroy_active_session(session_key) {
            Ok(resource) => Ok(resource),
            Err(AppError::SessionNotFound) => {
                if self.remove_hibernated_session_file(session_key).await? {
                    let resource = SessionResource::ZERO;

                    tracing::info!("hibernated session destroyed; resource={resource:?}");

                    Ok(resource)
                } else {
                    Err(AppError::SessionNotFound)
                }
            }
            Err(err) => Err(err),
        }
    }

    pub fn subscribe_session_response(&self, session_key: SessionKey) -> Result<SessionResponseReceiver, AppError> {
        let response_receiver = self.sessions.with_touch(&session_key, |session| {
            session.response_sender.subscribe()
        })
            .ok_or(AppError::SessionNotFound)?;

        Ok(response_receiver)
    }

    pub async fn hibernate_all_sessions(&self) -> Result<(), AppError> {
        for session_key in self.sessions.keys() {
            if let Err(err) = self.hibernate_active_session(session_key).await {
                tracing::warn!("failed to hibernate session: sid={session_key}, err={err}; skipping");
            }
        }

        Ok(())
    }
}
