use crate::app_state::WorkerPermit;
use crate::stream_response_sender::StreamResponseSender;
use anyhow::{anyhow, bail};
use dashmap::DashMap;
use mintaka::config::Config;
use mintaka::game_agent::{BestMove, GameAgent};
use mintaka::protocol::command::Command;
use mintaka::protocol::message::{GameResult, Message, MessageSender};
use mintaka::protocol::response::{Response, ResponseSender};
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::memo::hash_key::HashKey;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::SemaphorePermit;
use uuid::Uuid;

const SESSION_NOT_FOUND_MESSAGE: &str = "session not found";
const AGENT_IN_GAME_MESSAGE: &str = "agent in searching";
const AGENT_NOT_IN_GAME_MESSAGE: &str = "agent not in searching";

pub enum AgentState {
    Agent(GameAgent),
    Permit(WorkerPermit)
}

pub struct Session {
    state: AgentState,
    abort_handle: Arc<AtomicBool>,
}

impl Session {

    pub fn new(config: Config, board: Board, history: History) -> Self {
        Self {
            state: AgentState::Agent(GameAgent::from_state(config, board, history)),
            abort_handle: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn command(&mut self, command: Command) -> anyhow::Result<Option<GameResult>> {
        let game_agent = match &mut self.state {
            AgentState::Agent(agent) => agent,
            AgentState::Permit(_) => bail!(AGENT_IN_GAME_MESSAGE),
        };

        let (tx, rx) = std::sync::mpsc::channel();

        game_agent.command(&MessageSender::new(tx), command)
            .map_err(anyhow::Error::msg)?;

        let game_result = rx.try_iter().find_map(|message|
            match message {
                Message::Finished(result) => Some(result),
                _ => None
            }
        );

        Ok(game_result)
    }

    pub fn launch(
        &mut self,
        response_sender: StreamResponseSender<Response>,
        result_sender: StreamResponseSender<(GameAgent, BestMove)>,
        worker_permit: WorkerPermit,
    ) -> anyhow::Result<()> {
        let AgentState::Agent(mut game_agent)
            = std::mem::replace(&mut self.state, AgentState::Permit(worker_permit))
                else { bail!(AGENT_IN_GAME_MESSAGE) };

        self.abort_handle.store(false, Ordering::Relaxed);
        let abort_flag = self.abort_handle.clone();

        tokio::task::spawn_blocking(move || {
            let best_move = game_agent.launch(response_sender, abort_flag);

            result_sender.send(game_agent, best_move);
        });

        Ok(())
    }

    pub fn abort(&self) -> anyhow::Result<()> {
        match &self.state {
            AgentState::Agent(_) => bail!(AGENT_NOT_IN_GAME_MESSAGE),
            AgentState::Permit(_) => {
                self.abort_handle.store(true, Ordering::Relaxed);
            }
        }

        Ok(())
    }

    pub fn recover(&mut self, game_agent: GameAgent) -> anyhow::Result<()> {
        let permit = match std::mem::replace(&mut self.state, AgentState::Agent(game_agent)) {
            AgentState::Permit(permit) => permit,
            AgentState::Agent(prev_agent) => {
                self.state = AgentState::Agent(prev_agent);

                bail!(AGENT_NOT_IN_GAME_MESSAGE);
            }
        };

        permit.release();

        Ok(())
    }

    pub fn board_hash_key(&self) -> anyhow::Result<HashKey> {
        let hash_key = match &self.state {
            AgentState::Agent(agent) => agent.state.board.hash_key,
            AgentState::Permit(_) => bail!(AGENT_IN_GAME_MESSAGE),
        };

        Ok(hash_key)
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionKey(Uuid);

#[derive(Clone)]
pub enum SessionResponse {
    Response(Response),
    BestMove(BestMove),
}

pub enum SessionMessage {
    Status
}

#[derive(Default)]
pub struct Sessions {
    map: DashMap<SessionKey, Session>,
}

impl Sessions {

    pub fn new_session(&mut self, config: Config, board: Board, history: History) -> SessionKey {
        let session_key = SessionKey(Uuid::new_v4());

        let session = Session::new(config, board, history);

        self.map.insert(session_key, session);

        session_key
    }

    pub fn command_session(
        &mut self,
        session_key: SessionKey,
        command: Command,
    ) -> anyhow::Result<Option<GameResult>> {
        let mut session = self.map.get_mut(&session_key)
            .ok_or(anyhow!(SESSION_NOT_FOUND_MESSAGE))?;

        session.command(command)
    }

    pub fn launch_session(
        &mut self,
        session_key: SessionKey,
        response_sender: tokio::sync::mpsc::UnboundedSender<(SessionKey, SessionResponse)>,
        worker_permit: SemaphorePermit,
    ) -> anyhow::Result<()> {
        let mut session = self.map.get_mut(&session_key)
            .ok_or(anyhow!(SESSION_NOT_FOUND_MESSAGE))?;

        // recover session logic
        // best-move into session-response
        // response-stream into session-response

        Ok(())
    }

    pub fn abort_session(&mut self, session_key: SessionKey) -> anyhow::Result<()> {
        let session = self.map.get(&session_key)
            .ok_or(anyhow!(SESSION_NOT_FOUND_MESSAGE))?;

        session.abort()?;

        Ok(())
    }

    pub fn destroy_session(&mut self, session_key: SessionKey) -> anyhow::Result<()> {
        let (_, session) = self.map.remove(&session_key)
            .ok_or(anyhow!(SESSION_NOT_FOUND_MESSAGE))?;

        session.abort()?;

        Ok(())
    }

}
