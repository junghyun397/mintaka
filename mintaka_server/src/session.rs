use crate::stream_response_sender::StreamResponseSender;
use anyhow::{anyhow, bail};
use dashmap::DashMap;
use mintaka::config::Config;
use mintaka::game_agent::{BestMove, GameAgent};
use mintaka::protocol::command::Command;
use mintaka::protocol::message::Message;
use mintaka::protocol::response::{Response, ResponseSender};
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::memo::hash_key::HashKey;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;

const AGENT_IN_GAME_MESSAGE: &str = "agent in searching";
const AGENT_NOT_IN_GAME_MESSAGE: &str = "agent not in searching";

pub struct Session {
    game_agent: Option<GameAgent>,
    abort_handle: Arc<AtomicBool>,
}

impl Session {

    pub fn new(config: Config, board: Board, history: History) -> Self {
        Self {
            game_agent: Some(GameAgent::from_state(config, board, history)),
            abort_handle: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn command(&mut self, command: Command) -> anyhow::Result<std::sync::mpsc::Receiver<Message>> {
        let game_agent = self.game_agent.as_mut()
            .ok_or(anyhow!(AGENT_IN_GAME_MESSAGE))?;

        let (tx, rx) = std::sync::mpsc::channel();

        game_agent.command(&tx, command).map_err(anyhow::Error::msg)?;

        Ok(rx)
    }

    pub fn launch(&mut self, result_sender: tokio::sync::mpsc::Sender<(GameAgent, BestMove)>) -> UnboundedReceiverStream<Response> {
        let (response_sender, response_stream) = {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            (StreamResponseSender::new(tx), UnboundedReceiverStream::new(rx))
        };

        self.abort_handle.store(false, Ordering::Relaxed);

        let abort_flag = self.abort_handle.clone();
        let mut game_agent = std::mem::take(&mut self.game_agent).unwrap();

        tokio::task::spawn_blocking(move || {
            let best_move = game_agent.launch(response_sender, abort_flag);

            result_sender.send((game_agent, best_move)).unwrap();
        });

        response_stream
    }

    pub fn abort(&self) -> anyhow::Result<()> {
        if self.game_agent.is_some() {
            bail!(AGENT_NOT_IN_GAME_MESSAGE);
        }

        self.abort_handle.store(true, Ordering::Relaxed);

        Ok(())
    }

    pub fn recover(&mut self, game_agent: GameAgent) {
        self.game_agent = Some(game_agent);
    }

    pub fn board_hash_key(&self) -> anyhow::Result<HashKey> {
        let hash_key = self.game_agent.as_ref()
            .ok_or(anyhow!(AGENT_IN_GAME_MESSAGE))?
            .state.board.hash_key;

        Ok(hash_key)
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionKey(Uuid);

#[derive(Default)]
pub struct Sessions {
    map: DashMap<Uuid, Session>,
}

impl Sessions {

    pub fn new_session(&mut self, config: Config, board: Board, history: History) -> SessionKey {
        let session_key = SessionKey(Uuid::new_v4());

        let session = Session::new(config, board, history);

        self.map.insert(session_key.0, session);

        session_key
    }

    pub fn abort_session(&mut self, session_key: SessionKey) -> anyhow::Result<()> {
        let session = self.map.get(&session_key.0)
            .ok_or(anyhow!("session not found"))?;

        session.abort()?;

        Ok(())
    }

}
