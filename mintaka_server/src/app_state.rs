pub(crate) use crate::app_error::AppError;
use crate::preference::Preference;
use crate::session::{Session, SessionCommandResponse, SessionData, SessionKey, SessionResponse, SessionResultResponse, SessionStatus};
use crate::stream_response_sender::StreamSessionResponseSender;
use dashmap::DashMap;
use mintaka::config::Config;
use mintaka::game_agent::BestMove;
use mintaka::protocol::command::Command;
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::utils::byte_size::ByteSize;
use std::num::NonZeroU32;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tower_http::cors::{Any, CorsLayer};
use mintaka::state::GameState;

pub struct WorkerPermit(OwnedSemaphorePermit);

impl WorkerPermit {
    pub fn release(self) {
        drop(self.0);
    }
}

pub struct MemoryPermit(OwnedSemaphorePermit);

pub struct SessionResource {
    workers: NonZeroU32,
    running_time: Duration,
}

pub struct AppState {
    sessions: Arc<DashMap<SessionKey, Session>>,
    session_streams: Arc<DashMap<SessionKey, (UnboundedSender<SessionResponse>, Option<UnboundedReceiverStream<SessionResponse>>)>>,
    hibernation_queue: Arc<RwLock<Vec<SessionKey>>>,
    worker_resource: Arc<Semaphore>,
    memory_resource: Arc<Semaphore>,
    pub preference: Preference,
    session_cleanup_task: Option<tokio::task::AbortHandle>,
}

impl AppState {

    pub fn new(preference: Preference) -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            session_streams: Arc::new(DashMap::new()),
            hibernation_queue: Arc::new(RwLock::new(vec![])),
            worker_resource: Arc::new(Semaphore::new(preference.cores)),
            memory_resource: Arc::new(Semaphore::new(preference.memory_limit.mib() as usize)),
            preference,
            session_cleanup_task: None,
        }
    }

    pub fn available_workers(&self) -> usize {
        self.worker_resource.available_permits()
    }

    pub async fn acquire_workers(&self, workers: u32) -> WorkerPermit {
        WorkerPermit(self.worker_resource.clone().acquire_many_owned(workers).await.unwrap())
    }

    pub fn available_memory(&self) -> ByteSize {
        ByteSize::from_mib(self.memory_resource.available_permits() as u64)
    }

    pub async fn acquire_memory(&self, memory_size: ByteSize, force_acquire: bool) -> MemoryPermit {
        let available = ByteSize::from_mib(self.memory_resource.available_permits() as u64);

        if memory_size > available && force_acquire {
            let mut acquired_memory = ByteSize::ZERO;

            while acquired_memory >= memory_size {
                // get the oldest key from session_index
                // hibernates the oldest session and append acquired_memory
                acquired_memory += ByteSize::ZERO;
            }
        }

        MemoryPermit(self.memory_resource.clone().acquire_many_owned(memory_size.mib() as u32).await.unwrap())
    }

    pub async fn check_session(&self, session_key: SessionKey) -> Result<SessionStatus, AppError> {
        match self.sessions.get(&session_key) {
            Some(session) => Ok(session.status()),
            None => self.check_hibernated_session(session_key)
                .await
                .then_some(SessionStatus::Hibernating)
                .ok_or(AppError::SessionNotFound),
        }
    }

    pub async fn new_session(&self, config: Config, game_state: GameState) -> Result<SessionKey, AppError> {
        if Some(config) > self.preference.max_config {
            return Err(AppError::InvalidConfig);
        }

        let session_key = SessionKey::new_random();

        let memory_permit = self.acquire_memory(config.tt_size, true).await;

        let session = Session::new(config, game_state, None, memory_permit, false);

        self.sessions.insert(session_key, session);

        let (session_stream_sender, session_stream) = {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            (tx, UnboundedReceiverStream::new(rx))
        };

        self.session_streams.insert(session_key, (session_stream_sender, Some(session_stream)));

        tracing::info!("new session created: sid={session_key}");

        Ok(session_key)
    }

    pub async fn state_session(&self, session_key: SessionKey) -> Result<GameState, AppError> {
        self.sessions.get(&session_key)
            .ok_or(AppError::SessionNotFound)
            ?.game_state()
    }

    pub async fn hibernate_session(&self, session_key: SessionKey) -> Result<(), AppError> {
        let (_, session) = self.sessions.remove(&session_key)
            .ok_or(AppError::SessionNotFound)?;

        let encoded = tokio::task::spawn_blocking(move || rmp_serde::to_vec(&session))
            .await
            .map_err(AppError::from_general_error)?
            .map_err(AppError::from_general_error)?;

        let mut file = tokio::fs::File::create(format!("{}/{session_key}", self.preference.sessions_directory))
            .await
            .map_err(|_| AppError::SessionFileAlreadyExists)?;

        file.write_all(&encoded)
            .await
            .map_err(AppError::from_general_error)?;

        file.flush()
            .await
            .map_err(AppError::from_general_error)?;

        tracing::info!("session hibernated: sid={session_key}");

        Ok(())
    }

    pub async fn check_hibernated_session(&self, session_key: SessionKey) -> bool {
        tokio::fs::try_exists(format!("{}/{session_key}", self.preference.sessions_directory))
            .await
            .unwrap_or(false)
    }

    pub async fn wakeup_session(&self, session_key: SessionKey) -> Result<(), AppError> {
        let buf = tokio::fs::read(format!("{}/{session_key}", self.preference.sessions_directory))
            .await
            .map_err(|_| AppError::SessionFileNotFound)?;

        let session_data: SessionData = rmp_serde::from_slice(&buf)
            .map_err(AppError::from_general_error)?;

        let memory_permit = self.acquire_memory(session_data.agent.config.tt_size, true).await;

        let session = tokio::task::spawn_blocking(move || Session::from_data(session_data, memory_permit))
            .await
            .map_err(AppError::from_general_error)?;

        self.sessions.insert(session_key, session);

        let (session_stream_sender, session_stream) = {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            (tx, UnboundedReceiverStream::new(rx))
        };

        self.session_streams.insert(session_key, (session_stream_sender, Some(session_stream)));

        tracing::info!("session woken up: sid={session_key}");

        Ok(())
    }

    pub fn command_session(
        &self,
        session_key: SessionKey,
        command: Command,
    ) -> Result<SessionCommandResponse, AppError> {
        let mut session = self.sessions.get_mut(&session_key)
            .ok_or(AppError::SessionNotFound)?;
        tracing::info!("command received");

        session.command(command)
    }

    pub async fn launch_session(
        &self,
        session_key: SessionKey,
    ) -> Result<(), AppError> {
        let mut session = self.sessions.get_mut(&session_key)
            .ok_or(AppError::SessionNotFound)?;

        let session_stream_pair = self.session_streams.get(&session_key)
            .ok_or(AppError::SessionInComputing)?;

        let worker_permit = self.acquire_workers(session.required_workers()?).await;

        let (result_tx, result_rx) = tokio::sync::oneshot::channel();
        let (session_response_sender, _) = session_stream_pair.value();

        session.launch(
            StreamSessionResponseSender::new(session_response_sender.clone()),
            result_tx,
            worker_permit
        )?;

        let sessions = self.sessions.clone();
        let session_response_sender = session_response_sender.clone();

        tokio::spawn(async move {
            match result_rx.await {
                Ok(SessionResultResponse { game_agent, best_move, .. }) => {
                    if let Some(mut session) = sessions.get_mut(&session_key) {
                        session.store_best_move(best_move);
                        session.restore(game_agent).unwrap();
                    }

                    let _ = session_response_sender.send(SessionResponse::BestMove(best_move));
                }
                Err(err) => {
                    eprintln!("{err}")
                }
            }
        });

        Ok(())
    }

    pub fn abort_session(&self, session_key: SessionKey) -> Result<(), AppError> {
        let session = self.sessions.get(&session_key)
            .ok_or(AppError::SessionNotFound)?;

        if session.status() == SessionStatus::Idle {
            return Err(AppError::SessionIdle);
        }

        session.abort()?;

        Ok(())
    }

    pub fn get_session_result(&self, session_key: SessionKey) -> Result<BestMove, AppError> {
        self.sessions.get(&session_key)
            .ok_or(AppError::SessionNotFound)?
            .last_best_move()
            .ok_or(AppError::SessionNeverLaunched)
    }

    pub fn destroy_session(&self, session_key: SessionKey) -> Result<(), AppError> {
        let (_, session) = self.sessions.remove(&session_key)
            .ok_or(AppError::SessionNotFound)?;

        self.session_streams.remove(&session_key);

        session.abort()?;

        tracing::info!("session destroyed: sid={session_key}");

        Ok(())
    }

    pub fn acquire_session_stream(&self, session_key: SessionKey) -> Result<UnboundedReceiverStream<SessionResponse>, AppError> {
        let session_stream_receiver = self.session_streams.get_mut(&session_key)
            .ok_or(AppError::SessionNotFound)?
            .1.take()
            .ok_or(AppError::StreamAcquired)?;

        Ok(session_stream_receiver)
    }

    pub fn restore_session_stream(&self, session_key: SessionKey, session_stream_receiver: UnboundedReceiverStream<SessionResponse>) -> Result<(), AppError> {
        self.session_streams.get_mut(&session_key)
            .ok_or(AppError::StreamNotAcquired)?.1
            = Some(session_stream_receiver);

        Ok(())
    }

    pub async fn hibernate_all_sessions(&self) -> Result<(), AppError> {
        let session_keys: Vec<_> = self.sessions.iter()
            .map(|session| *session.key())
            .collect();

        let tasks = session_keys.into_iter()
            .map(|session_key| self.hibernate_session(session_key));

        futures_util::future::try_join_all(tasks).await?;

        Ok(())
    }

    pub async fn wakeup_all_sessions(&self) -> Result<(), AppError> {
        let sessions_directory = &self.preference.sessions_directory;

        let mut read_dir = tokio::fs::read_dir(sessions_directory).await
            .map_err(AppError::from_general_error)?;
        let mut session_keys = vec![];

        while let Some(entry) = read_dir.next_entry()
            .await
            .map_err(AppError::from_general_error)?
        {
            if let Ok(session_key) = SessionKey::from_str(&entry.file_name().to_string_lossy()) {
                session_keys.push(session_key);
            }
        }

        let tasks = session_keys.into_iter()
            .map(|session_key| self.wakeup_session(session_key));

        futures_util::future::try_join_all(tasks).await?;

        Ok(())
    }

    pub fn spawn_session_cleanup(&mut self) {
        assert!(self.session_cleanup_task.is_none());

        let sessions = self.sessions.clone();

        let join_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;
                let now = Instant::now();

                sessions.retain(|_, session| !session.is_expired(now));
            }
        });

        self.session_cleanup_task = Some(join_handle.abort_handle());
    }

}

impl Drop for AppState {
    fn drop(&mut self) {
        if let Some(abort_handle) = self.session_cleanup_task.take() {
            abort_handle.abort();
        }
    }
}
