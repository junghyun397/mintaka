use crate::app_state::{AppError, WorkerPermit};
use crate::stream_response_sender::StreamSessionResponseSender;
use mintaka::config::Config;
use mintaka::game_agent::{BestMove, GameAgent};
use mintaka::game_state::GameState;
use mintaka::movegen::movegen_window::MovegenWindow;
use mintaka::protocol::command::Command;
use mintaka::protocol::message::GameResult;
use mintaka::protocol::response::{Response, ResponseSender};
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::memo::hash_key::HashKey;
use serde::ser::SerializeStruct;
use serde::{ser, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionKey(Uuid);

impl Display for SessionKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Hash for SessionKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.0.as_bytes())
    }
}

impl From<Uuid> for SessionKey {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl FromStr for SessionKey {
    type Err = AppError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::from_str(source).map_err(|_| AppError::InvalidSessionId)?))
    }
}

impl SessionKey {
    pub fn new_random() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SessionResponse {
    Response(Response),
    BestMove(BestMove),
    Terminate,
}

pub struct SessionResultResponse {
    pub game_agent: GameAgent,
    pub best_move: BestMove,
}

impl Debug for SessionResultResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.game_agent.state.history, self.best_move.pos)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionCommandResponse {
    board_hash: HashKey,
    game_result: Option<GameResult>,
}

pub enum AgentState {
    Agent(GameAgent),
    Permit(WorkerPermit)
}

pub struct Session {
    state: AgentState,
    best_move: Option<BestMove>,
    abort_handle: Arc<AtomicBool>,
    time_to_live: Option<Duration>,
    last_active: Instant,
}

impl Session {

    pub fn new(config: Config, board: Board, history: History, time_to_live: Option<Duration>) -> Self {
        let game_state = GameState {
            movegen_window: MovegenWindow::from(&board.hot_field),
            board,
            history,
        };

        Self {
            state: AgentState::Agent(GameAgent::from_state(config, game_state)),
            best_move: None,
            abort_handle: Arc::new(AtomicBool::new(false)),
            time_to_live,
            last_active: Instant::now(),
        }
    }

    pub fn command(&mut self, command: Command) -> Result<SessionCommandResponse, AppError> {
        let game_agent = match &mut self.state {
            AgentState::Agent(agent) => agent,
            AgentState::Permit(_) => return Err(AppError::SessionInComputing),
        };

        let game_result = game_agent.command(command)
            .map_err(AppError::GameError)?;

        Ok(SessionCommandResponse {
            board_hash: game_agent.state.board.hash_key,
            game_result,
        })
    }

    pub fn required_workers(&self) -> Result<u32, AppError> {
        match &self.state {
            AgentState::Agent(agent) => Ok(agent.config.workers),
            AgentState::Permit(_) => Err(AppError::SessionInComputing),
        }
    }

    pub fn launch(
        &mut self,
        response_sender: StreamSessionResponseSender,
        result_sender: tokio::sync::oneshot::Sender<SessionResultResponse>,
        worker_permit: WorkerPermit,
    ) -> Result<(), AppError> {
        let AgentState::Agent(mut game_agent)
            = std::mem::replace(&mut self.state, AgentState::Permit(worker_permit))
                else { return Err(AppError::SessionInComputing) };

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

    pub fn is_computing(&self) -> bool {
        match &self.state {
            AgentState::Agent(_) => false,
            AgentState::Permit(_) => true,
        }
    }

    pub fn abort(&self) -> Result<(), AppError> {
        match &self.state {
            AgentState::Agent(_) => Err(AppError::SessionIdle),
            AgentState::Permit(_) => {
                self.abort_handle.store(true, Ordering::Relaxed);
                Ok(())
            }
        }
    }

    pub fn store_best_move(&mut self, best_move: BestMove) {
        self.best_move = Some(best_move);
    }

    pub fn last_best_move(&self) -> Option<BestMove> {
        self.best_move
    }

    pub fn restore(&mut self, game_agent: GameAgent) -> Result<(), AppError> {
        let permit = match std::mem::replace(&mut self.state, AgentState::Agent(game_agent)) {
            AgentState::Permit(permit) => permit,
            AgentState::Agent(prev_agent) => {
                self.state = AgentState::Agent(prev_agent);

                return Err(AppError::SessionIdle);
            }
        };

        permit.release();

        Ok(())
    }

    pub fn board_hash_key(&self) -> Result<HashKey, AppError> {
        let hash_key = match &self.state {
            AgentState::Agent(agent) => agent.state.board.hash_key,
            AgentState::Permit(_) => return Err(AppError::SessionInComputing),
        };

        Ok(hash_key)
    }

    fn touch_last_active(&mut self) {
        self.last_active = Instant::now();
    }

    pub fn is_expired(&self, now: Instant) -> bool {
        match self.time_to_live {
            Some(time_to_live) => now.duration_since(self.last_active) > time_to_live,
            None => false,
        }
    }

}

impl Drop for Session {
    fn drop(&mut self) {
        if let AgentState::Permit(_) = &self.state {
            self.abort_handle.store(true, Ordering::Relaxed);
        }
    }
}

impl Serialize for Session {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let AgentState::Agent(agent) = &self.state else {
            return Err(ser::Error::custom("session is not idle"));
        };

        let mut state = serializer.serialize_struct("Session", 3)?;
        state.serialize_field("agent", &agent)?;
        state.serialize_field("best_move", &self.best_move)?;
        state.serialize_field("time_to_live", &self.time_to_live)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Session {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        #[derive(Deserialize)]
        struct SessionData {
            agent: GameAgent,
            best_move: Option<BestMove>,
            time_to_live: Option<Duration>,
        }

        let data = SessionData::deserialize(deserializer)?;

        Ok(Self {
            state: AgentState::Agent(data.agent),
            best_move: data.best_move,
            abort_handle: Arc::new(AtomicBool::new(false)),
            time_to_live: data.time_to_live,
            last_active: Instant::now(),
        })
    }
}
