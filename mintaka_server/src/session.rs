use crate::app_state::WorkerPermit;
use crate::stream_response_sender::StreamSessionResponseSender;
use anyhow::bail;
use mintaka::config::Config;
use mintaka::game_agent::{BestMove, GameAgent};
use mintaka::protocol::command::Command;
use mintaka::protocol::message::{GameResult, Message, MessageSender};
use mintaka::protocol::response::{Response, ResponseSender};
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::memo::hash_key::HashKey;
use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use uuid::Uuid;

const AGENT_IN_GAME_MESSAGE: &str = "agent in searching";
const AGENT_NOT_IN_GAME_MESSAGE: &str = "agent not in searching";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionKey(Uuid);

impl SessionKey {
    pub fn new_random() -> Self {
        Self(Uuid::new_v4())
    }
}

pub enum SessionResponse {
    Response(Response),
    BestMove(BestMove),
    Terminate
}

pub struct SessionResultResponse {
    pub game_agent: GameAgent,
    pub best_move: BestMove,
}

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
        response_sender: StreamSessionResponseSender,
        result_sender: tokio::sync::oneshot::Sender<SessionResultResponse>,
        worker_permit: WorkerPermit,
    ) -> anyhow::Result<()> {
        let AgentState::Agent(mut game_agent)
            = std::mem::replace(&mut self.state, AgentState::Permit(worker_permit))
                else { bail!(AGENT_IN_GAME_MESSAGE) };

        self.abort_handle.store(false, Ordering::Relaxed);
        let abort_flag = self.abort_handle.clone();

        tokio::task::spawn_blocking(async move || {
            let best_move = game_agent.launch(response_sender, abort_flag);

            result_sender
                .send(SessionResultResponse { game_agent, best_move })
                .unwrap();
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

impl Debug for SessionResultResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.game_agent.state.history, self.best_move.pos)
    }
}
