pub(crate) use crate::app_error::AppError;
use crate::preference::Preference;
use crate::session::{Session, SessionCommandResponse, SessionKey, SessionResponse, SessionResultResponse};
use crate::stream_response_sender::StreamSessionResponseSender;
use dashmap::DashMap;
use mintaka::config::Config;
use mintaka::game_agent::{BestMove, ComputingResource};
use mintaka::protocol::command::Command;
use rusty_renju::board::Board;
use rusty_renju::history::History;
use serde_json::json;
use std::num::NonZeroU32;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::info;

pub struct WorkerPermit(OwnedSemaphorePermit);

impl WorkerPermit {

    pub fn release(self) {
        drop(self.0)
    }

}

pub struct SessionResource {
    workers: NonZeroU32,
    running_time: Duration,
}

pub struct AppState {
    sessions: Arc<DashMap<SessionKey, Session>>,
    session_streams: Arc<DashMap<SessionKey, (UnboundedSender<SessionResponse>, Option<UnboundedReceiverStream<SessionResponse>>)>>,
    sem: Arc<Semaphore>,
    pub preference: Preference,
    session_cleanup_task: Option<tokio::task::AbortHandle>,
}

impl AppState {

    pub fn new(preference: Preference) -> Self {
        let sem = Arc::new(Semaphore::new(preference.cores));

        Self {
            sessions: Arc::new(DashMap::new()),
            session_streams: Arc::new(DashMap::new()),
            sem,
            preference,
            session_cleanup_task: None,
        }
    }

    pub fn available_workers(&self) -> usize {
        self.sem.available_permits()
    }

    pub async fn acquire_workers(&self, workers: u32) -> WorkerPermit {
        WorkerPermit(self.sem.clone().acquire_many_owned(workers).await.unwrap())
    }

    pub fn new_session(&self, config: Config, board: Board, history: History) -> SessionKey {
        let session_key = SessionKey::new_random();

        let session = Session::new(config, board, history, None);

        self.sessions.insert(session_key, session);

        let (session_stream_sender, session_stream) = {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            (tx, UnboundedReceiverStream::new(rx))
        };

        self.session_streams.insert(session_key, (session_stream_sender, Some(session_stream)));

        info!("new session created: sid={session_key}");

        session_key
    }

    pub async fn hibernate_session(&self, session_key: SessionKey, sessions_directory: &str) -> Result<(), AppError> {
        let (_, session) = self.sessions.remove(&session_key)
            .ok_or(AppError::SessionNotFound)?;

        let encoded = tokio::task::spawn_blocking(move || rmp_serde::to_vec(&session))
            .await
            .map_err(AppError::from)?
            .map_err(AppError::from)?;

        let mut file = tokio::fs::File::create(format!("{sessions_directory}/{session_key}"))
            .await
            .map_err(|_| AppError::SessionFileAlreadyExists)?;

        file.write_all(&encoded).await?;
        file.flush().await?;

        info!("session hibernated: sid={session_key}");

        Ok(())
    }

    pub async fn wakeup_session(&self, session_key: SessionKey, session_directory: &str) -> Result<(), AppError> {
        let buf = tokio::fs::read(format!("{session_directory}/{session_key}"))
            .await
            .map_err(|_| AppError::SessionFileNotFound)?;

        let session = tokio::task::spawn_blocking(move || rmp_serde::from_slice(&buf))
            .await
            .map_err(AppError::from)?
            .map_err(AppError::from)?;

        self.sessions.insert(session_key, session);

        let (session_stream_sender, session_stream) = {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            (tx, UnboundedReceiverStream::new(rx))
        };

        self.session_streams.insert(session_key, (session_stream_sender, Some(session_stream)));

        info!("session woken up: sid={session_key}");

        Ok(())
    }

    pub fn command_session(
        &self,
        session_key: SessionKey,
        command: Command,
    ) -> Result<SessionCommandResponse, AppError> {
        let mut session = self.sessions.get_mut(&session_key)
            .ok_or(AppError::SessionNotFound)?;

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
                Ok(SessionResultResponse { game_agent, best_move }) => {
                    if let Some(mut session) = sessions.get_mut(&session_key) {
                        session.store_best_move(best_move);
                        session.restore(game_agent).unwrap();
                    }

                    session_response_sender.send(SessionResponse::BestMove(best_move)).unwrap();
                }
                Err(_) => {}
            }
        });

        info!("session launched: sid={session_key}");

        Ok(())
    }

    pub fn abort_session(&self, session_key: SessionKey) -> Result<(), AppError> {
        let session = self.sessions.get(&session_key)
            .ok_or(AppError::SessionNotFound)?;

        if !session.is_computing() {
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

        info!("session destroyed: sid={session_key}");

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
        let sessions_directory = &self.preference.sessions_directory;

        let session_keys: Vec<_> = self.sessions.iter()
            .map(|session| *session.key())
            .collect();

        let tasks = session_keys.into_iter()
            .map(|session_key| self.hibernate_session(session_key, sessions_directory));

        futures_util::future::try_join_all(tasks).await?;

        Ok(())
    }

    pub async fn wakeup_all_sessions(&self) -> Result<(), AppError> {
        let sessions_directory = &self.preference.sessions_directory;

        let mut read_dir = tokio::fs::read_dir(sessions_directory).await?;
        let mut session_keys = vec![];

        while let Some(entry) = read_dir.next_entry().await? {
            if let Ok(session_key) = SessionKey::from_str(&entry.file_name().to_string_lossy()) {
                session_keys.push(session_key);
            }
        }

        let tasks = session_keys.into_iter()
            .map(|session_key| self.wakeup_session(session_key, sessions_directory));

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
