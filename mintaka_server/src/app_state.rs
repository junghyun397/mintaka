pub(crate) use crate::app_error::AppError;
use crate::preference::Preference;
use crate::session::{Session, SessionKey, SessionResponse, SessionResultResponse};
use crate::stream_response_sender::StreamSessionResponseSender;
use anyhow::anyhow;
use dashmap::DashMap;
use mintaka::config::Config;
use mintaka::game_agent::BestMove;
use mintaka::protocol::command::Command;
use mintaka::protocol::message::GameResult;
use rusty_renju::board::Board;
use rusty_renju::history::History;
use std::num::NonZeroU32;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_stream::wrappers::UnboundedReceiverStream;

pub struct WorkerPermit(OwnedSemaphorePermit);

impl WorkerPermit {

    pub fn release(self) {
        drop(self.0)
    }

}

pub struct AppState {
    sessions: Arc<DashMap<SessionKey, Session>>,
    session_streams: Arc<DashMap<SessionKey, (UnboundedSender<SessionResponse>, Option<UnboundedReceiverStream<SessionResponse>>)>>,
    sem: Arc<Semaphore>,
    preference: Preference,
}

impl AppState {

    pub fn new(preference: Preference) -> Self {
        let sem = Arc::new(Semaphore::new(preference.cores));

        Self {
            sessions: Arc::new(DashMap::new()),
            session_streams: Arc::new(DashMap::new()),
            sem,
            preference,
        }
    }

    pub fn available_workers(&self) -> usize {
        self.sem.available_permits()
    }

    pub async fn acquire_workers(&self, workers: NonZeroU32) -> WorkerPermit {
        WorkerPermit(self.sem.clone().acquire_many_owned(workers.get()).await.unwrap())
    }

    pub fn new_session(&self, config: Config, board: Board, history: History) -> SessionKey {
        let session_key = SessionKey::new_random();

        let session = Session::new(config, board, history);

        self.sessions.insert(session_key, session);

        let (session_stream_sender, session_stream_rx) = {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            (tx, UnboundedReceiverStream::new(rx))
        };

        self.session_streams.insert(session_key, (session_stream_sender, Some(session_stream_rx)));

        session_key
    }

    pub fn command_session(
        &self,
        session_key: SessionKey,
        command: Command,
    ) -> anyhow::Result<Option<GameResult>, AppError> {
        let mut session = self.sessions.get_mut(&session_key)
            .ok_or(AppError::SessionNotFound)?;

        session.command(command)
    }

    pub async fn launch_session(
        &self,
        session_key: SessionKey,
    ) -> anyhow::Result<(), AppError> {
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

        Ok(())
    }

    pub fn abort_session(&self, session_key: SessionKey) -> anyhow::Result<(), AppError> {
        let session = self.sessions.get(&session_key)
            .ok_or(AppError::SessionNotFound)?;

        if !session.is_computing() {
            return Err(AppError::SessionIdle);
        }

        session.abort()?;

        Ok(())
    }

    pub fn get_session_result(&self, session_key: SessionKey) -> anyhow::Result<BestMove, AppError> {
        self.sessions.get(&session_key)
            .ok_or(AppError::SessionNotFound)?
            .last_best_move()
            .ok_or(AppError::SessionNeverLaunched)
    }

    pub fn destroy_session(&self, session_key: SessionKey) -> anyhow::Result<(), AppError> {
        let (_, session) = self.sessions.remove(&session_key)
            .ok_or(AppError::SessionNotFound)?;

        self.session_streams.remove(&session_key);

        session.abort()?;

        Ok(())
    }

    pub fn acquire_session_stream(&self, session_key: SessionKey) -> anyhow::Result<UnboundedReceiverStream<SessionResponse>, AppError> {
        let session_stream_receiver = self.session_streams.get_mut(&session_key)
            .ok_or(AppError::SessionNotFound)?
            .1.take()
            .ok_or(AppError::StreamAcquired)?;

        Ok(session_stream_receiver)
    }

    pub fn restore_session_stream(&self, session_key: SessionKey, session_stream_receiver: UnboundedReceiverStream<SessionResponse>) -> anyhow::Result<()> {
        self.session_streams.get_mut(&session_key)
            .ok_or(anyhow!(AppError::StreamNotAcquired))?.1
            = Some(session_stream_receiver);

        Ok(())
    }

    pub async fn hibernate_session(&self, _session_key: SessionKey) -> anyhow::Result<(), AppError> {
        todo!()
    }

    pub async fn wakeup_session(&self, _session_key: SessionKey) -> anyhow::Result<(), AppError> {
        todo!()
    }

}
