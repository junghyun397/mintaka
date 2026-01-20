pub(crate) use crate::app_error::AppError;
use crate::preference::Preference;
use crate::session::{Session, SessionData, SessionKey, SessionResponse, SessionResultResponse, SessionStatus, Sessions};
use crate::stream_response_sender::StreamSessionResponseSender;
use mintaka::config::Config;
use mintaka::protocol::command::Command;
use mintaka::state::GameState;
use rusty_renju::utils::byte_size::ByteSize;
use std::cmp::Reverse;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_stream::wrappers::UnboundedReceiverStream;
use mintaka::protocol::results::{BestMove, CommandResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct Configs {
    pub default_config: Config,
    pub max_config: Config,
    pub config: Config,
}

pub struct WorkerPermit(OwnedSemaphorePermit);

impl WorkerPermit {
    pub fn release(self) {
        drop(self.0);
    }
}

pub struct MemoryPermit(OwnedSemaphorePermit);

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

pub struct AppState {
    pub sessions: Arc<Sessions>,
    worker_resource: Arc<Semaphore>,
    memory_resource: Arc<Semaphore>,
    pub preference: Preference,
}

impl AppState {

    pub fn new(preference: Preference) -> Self {
        Self {
            sessions: Arc::new(Sessions::default()),
            worker_resource: Arc::new(Semaphore::new(preference.cores)),
            memory_resource: Arc::new(Semaphore::new(preference.memory_limit.mib() as usize)),
            preference,
        }
    }

    pub fn max_config(&self) -> Option<Config> {
        self.preference.max_config
    }

    pub fn available_workers(&self) -> usize {
        self.worker_resource.available_permits()
    }

    pub async fn acquire_workers(&self, workers: u32, timeout: Duration) -> Result<WorkerPermit, AppError> {
        let worker_permit = tokio::time::timeout(
            timeout,
            self.worker_resource.clone().acquire_many_owned(workers)
        )
            .await
            .map_err(|_| AppError::WorkerAcquireTimeout)?;

        Ok(WorkerPermit(worker_permit.unwrap()))
    }

    pub fn available_memory(&self) -> ByteSize {
        ByteSize::from_mib(self.memory_resource.available_permits() as u64)
    }

    pub async fn acquire_memory(&self, memory_size: ByteSize, force_acquire: bool) -> Result<MemoryPermit, AppError> {
        let available = ByteSize::from_mib(self.memory_resource.available_permits() as u64);

        if memory_size > available && force_acquire {
            let mut acquired_memory = ByteSize::ZERO;

            let mut eviction_queue = self.sessions.eviction_queue.lock().unwrap();

            while acquired_memory < memory_size
                && let Some(Reverse(entry)) = eviction_queue.pop()
            {
                match self.sessions.get(&entry.key)
                    .map(|session|
                        (session.last_active_seq == entry.last_active_seq, session.status() == SessionStatus::Idle)
                    )
                    .unwrap_or((false, false))
                {
                    (true, false) => eviction_queue.push(Reverse(entry)),
                    (true, true) => {
                        let freed_resource = self.destroy_session(entry.key)?;

                        acquired_memory += freed_resource.memory;
                    }
                    _ => {}
                }
            }
        }

        let memory_permit = self.memory_resource.clone()
            .acquire_many_owned(memory_size.mib() as u32)
            .await
            .unwrap();

        Ok(MemoryPermit(memory_permit))
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

    pub async fn new_session(&self, config: Option<Config>, game_state: GameState) -> Result<SessionKey, AppError> {
        if let Some(config) = config
            && let Some(max_config) = self.preference.max_config
            && config > max_config
        {
            return Err(AppError::InvalidConfig);
        }

        let config = config.unwrap_or(self.preference.default_config);

        let session_key = SessionKey::new_random();

        let memory_permit = self.acquire_memory(config.tt_size, true)
            .await?;

        let (tx, rx) = {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            (tx, UnboundedReceiverStream::new(rx))
        };

        let session = Session::new(config, game_state, None, memory_permit, tx, rx, false);

        self.sessions.insert(session_key, session);

        tracing::info!("session created; sid={session_key}");

        Ok(session_key)
    }

    pub fn configs_session(&self, session_key: SessionKey) -> Result<Configs, AppError> {
        let config = self.sessions.get(&session_key)
            .ok_or(AppError::SessionNotFound)?
            .game_agent()?
            .config;

        Ok(Configs {
            default_config: self.preference.default_config,
            max_config: self.preference.max_config.unwrap_or(Config::UNLIMITED_CONFIG),
            config,
        })
    }

    pub async fn hibernate_session(&self, session_key: SessionKey) -> Result<(), AppError> {
        let session = self.sessions.remove(&session_key)
            .ok_or(AppError::SessionNotFound)?;

        let file_path = format!(
            "{}/{session_key}_{}",
            self.preference.sessions_directory,
            session.live_until_epoch_secs()
        );

        let encoded = tokio::task::spawn_blocking(move || rmp_serde::to_vec(&session))
            .await
            .map_err(AppError::from_general_error)?
            .map_err(AppError::from_general_error)?;

        let mut file = tokio::fs::File::create(file_path)
            .await
            .map_err(|_| AppError::SessionFileAlreadyExists)?;

        file.write_all(&encoded)
            .await
            .map_err(AppError::from_general_error)?;

        file.flush()
            .await
            .map_err(AppError::from_general_error)?;

        tracing::info!("session hibernated");

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

        let memory_permit = self.acquire_memory(session_data.agent.config.tt_size, true)
            .await?;

        let (tx, rx) = {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            (tx, UnboundedReceiverStream::new(rx))
        };

        let session = tokio::task::spawn_blocking(move || Session::from_data(session_data, memory_permit, tx, rx))
            .await
            .map_err(AppError::from_general_error)?;

        self.sessions.insert(session_key, session);

        tracing::info!("session woken up");

        Ok(())
    }

    pub fn command_session(
        &self,
        session_key: SessionKey,
        command: Command,
    ) -> Result<CommandResult, AppError> {
        let mut session = self.sessions.get_mut_with_touch(&session_key)
            .ok_or(AppError::SessionNotFound)?;

        let command_message = command.to_brief_debug();

        let result = session.command(command);

        tracing::info!("session command executed; command={command_message}");

        result
    }

    pub async fn launch_session(
        &self,
        session_key: SessionKey,
        timeout: Duration,
    ) -> Result<(), AppError> {
        let mut session = self.sessions.get_mut_with_touch(&session_key)
            .ok_or(AppError::SessionNotFound)?;

        let worker_permit = self.acquire_workers(session.game_agent()?.config.workers, timeout).await?;

        let response_sender = session.response_sender.clone();

        let (result_tx, result_rx) = tokio::sync::oneshot::channel();

        session.launch(
            StreamSessionResponseSender::new(response_sender.clone()),
            result_tx,
            worker_permit
        )?;

        let sessions = self.sessions.clone();

        tokio::spawn(async move {
            match result_rx.await {
                Ok(SessionResultResponse { game_agent, best_move, .. }) => {
                    if let Some(mut session) = sessions.get_mut_with_touch(&session_key) {
                        session.store_best_move(best_move);
                        session.restore(game_agent).unwrap();
                    }

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
        let session = self.sessions.get_with_touch(&session_key)
            .ok_or(AppError::SessionNotFound)?;

        if session.status() == SessionStatus::Idle {
            return Err(AppError::SessionIdle);
        }

        session.abort()?;

        tracing::info!("session aborted");

        Ok(())
    }

    pub fn get_session_result(&self, session_key: SessionKey) -> Result<BestMove, AppError> {
        self.sessions.get(&session_key)
            .ok_or(AppError::SessionNotFound)?
            .last_best_move()
            .ok_or(AppError::SessionNeverLaunched)
    }

    pub fn destroy_session(&self, session_key: SessionKey) -> Result<SessionResource, AppError> {
        let session = self.sessions.remove(&session_key)
            .ok_or(AppError::SessionNotFound)?;

        let resource = (&session.game_agent()?.config).into();

        tracing::info!("session destroyed; resource={resource:?}");

        Ok(resource)
    }

    pub fn acquire_session_stream(&self, session_key: SessionKey) -> Result<UnboundedReceiverStream<SessionResponse>, AppError> {
        let session_stream_receiver = self.sessions.get_mut_with_touch(&session_key)
            .ok_or(AppError::SessionNotFound)?
            .response_receiver
            .take()
            .ok_or(AppError::StreamAlreadyAcquired)?;

        Ok(session_stream_receiver)
    }

    pub fn restore_session_stream(&self, session_key: SessionKey, session_stream_receiver: UnboundedReceiverStream<SessionResponse>) -> Result<(), AppError> {
        self.sessions.get_mut_with_touch(&session_key)
            .ok_or(AppError::StreamNotAcquired)?
            .response_receiver
            = Some(session_stream_receiver);

        Ok(())
    }

    pub async fn hibernate_all_sessions(&self) -> Result<(), AppError> {
        let tasks = self.sessions.keys().into_iter()
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
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            let Some((sid_str, _)) = file_name.split_once('_') else {
                continue;
            };

            let Ok(session_key) = SessionKey::from_str(sid_str) else {
                continue;
            };

            session_keys.push(session_key);
        }

        for session_key in session_keys {
            if let Err(err) = self.wakeup_session(session_key).await {
                tracing::warn!("failed to wake session: sid={}, err={}; skipping",session_key,err);
            }
        }

        Ok(())
    }

}
