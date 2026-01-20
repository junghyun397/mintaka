use crate::app_state::{AppError, MemoryPermit, WorkerPermit};
use crate::stream_response_sender::StreamSessionResponseSender;
use dashmap::DashMap;
use mintaka::config::{Config, SearchObjective};
use mintaka::game_agent::GameAgent;
use mintaka::protocol::command::Command;
use mintaka::protocol::results::{BestMove, CommandResult, GameResult};
use mintaka::protocol::response::Response;
use mintaka::state::GameState;
use rusty_renju::memo::hash_key::HashKey;
use serde::ser::SerializeStruct;
use serde::{ser, Deserialize, Serialize, Serializer};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::wrappers::UnboundedReceiverStream;
use typeshare::typeshare;
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

#[typeshare(serialized_as = "String")]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum SessionStatus {
    Idle,
    InComputing,
    Hibernating
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SessionResponse {
    Response(Response),
    BestMove(BestMove),
}

pub struct SessionResultResponse {
    pub game_agent: GameAgent,
    pub best_move: BestMove,
}

impl Debug for SessionResultResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.game_agent.state.history, self.best_move.best_move)
    }
}

pub enum AgentState {
    Agent(GameAgent),
    Permit(WorkerPermit)
}

pub type SessionResponseSender = UnboundedSender<SessionResponse>;
pub type SessionResponseReceiver = UnboundedReceiverStream<SessionResponse>;

pub struct Session {
    state: AgentState,
    pub response_sender: SessionResponseSender,
    pub response_receiver: Option<SessionResponseReceiver>,
    memory_permit: MemoryPermit,
    best_move: Option<BestMove>,
    abort_handle: Arc<AtomicBool>,
    time_to_live: Option<Duration>,
    pub persistence: bool,
    pub last_active: Instant,
    pub last_active_seq: u32,
}

#[derive(Deserialize)]
pub struct SessionData {
    pub agent: GameAgent,
    pub best_move: Option<BestMove>,
    pub time_to_live: Option<Duration>,
}

impl Session {

    pub fn new(config: Config, game_state: GameState, time_to_live: Option<Duration>, memory_permit: MemoryPermit, response_sender: SessionResponseSender, response_receiver: SessionResponseReceiver, persistence: bool) -> Self {
        Self {
            state: AgentState::Agent(GameAgent::from_state(config, game_state)),
            response_sender,
            response_receiver: Some(response_receiver),
            memory_permit,
            best_move: None,
            abort_handle: Arc::new(AtomicBool::new(false)),
            time_to_live,
            persistence,
            last_active: Instant::now(),
            last_active_seq: 0,
        }
    }

    pub fn from_data(data: SessionData, memory_permit: MemoryPermit, response_sender: SessionResponseSender, response_receiver: SessionResponseReceiver) -> Self {
        Self {
            state: AgentState::Agent(data.agent),
            response_sender,
            response_receiver: Some(response_receiver),
            memory_permit,
            best_move: data.best_move,
            abort_handle: Arc::new(AtomicBool::new(false)),
            time_to_live: data.time_to_live,
            persistence: true,
            last_active: Instant::now(),
            last_active_seq: 0,
        }
    }

    pub fn game_agent(&self) -> Result<&GameAgent, AppError> {
        match &self.state {
            AgentState::Agent(agent) => Ok(agent),
            AgentState::Permit(_) => Err(AppError::SessionInComputing),
        }
    }

    pub fn game_agent_mut(&mut self) -> Result<&mut GameAgent, AppError> {
        match &mut self.state {
            AgentState::Agent(agent) => Ok(agent),
            AgentState::Permit(_) => Err(AppError::SessionInComputing),
        }
    }

    pub fn touch_last_active(&mut self) {
        self.last_active = Instant::now();
        self.last_active_seq += 1;
    }

    pub fn command(&mut self, command: Command) -> Result<CommandResult, AppError> {
        let game_agent = self.game_agent_mut()?;

        game_agent.command(command)
            .map_err(AppError::GameError)
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

        tokio::task::spawn_blocking(move || {
            let best_move = game_agent.launch::<Instant>(
                SearchObjective::Best,
                response_sender,
                abort_flag
            );

            let _ = result_sender.send(SessionResultResponse { game_agent, best_move });

            tracing::info!("session finished")
        });

        Ok(())
    }

    pub fn status(&self) -> SessionStatus {
        match &self.state {
            AgentState::Agent(_) => SessionStatus::Idle,
            AgentState::Permit(_) => SessionStatus::InComputing,
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

    pub fn is_expired(&self, now: Instant) -> bool {
        match self.time_to_live {
            Some(time_to_live) => now.duration_since(self.last_active) > time_to_live,
            None => false,
        }
    }

    pub fn live_until_epoch_secs(&self) -> u64 {
        let live_time = SystemTime::now()
            - Instant::now().duration_since(self.last_active)
            + self.time_to_live.unwrap();

        live_time.duration_since(UNIX_EPOCH).unwrap().as_secs()
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

pub struct EvictionEntry {
    pub last_active: Instant,
    pub last_active_seq: u32,
    pub key: SessionKey,
}

impl PartialEq<Self> for EvictionEntry {
    fn eq(&self, other: &Self) -> bool {
        self.last_active == other.last_active
    }
}

impl Eq for EvictionEntry {}

impl PartialOrd<Self> for EvictionEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EvictionEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.last_active.cmp(&other.last_active)
    }
}

pub struct Sessions {
    pub map: Arc<DashMap<SessionKey, Session>>,
    pub eviction_queue: Arc<Mutex<BinaryHeap<Reverse<EvictionEntry>>>>,
}

impl Default for Sessions {
    fn default() -> Self {
        Self {
            map: Arc::new(DashMap::new()),
            eviction_queue: Arc::new(Mutex::new(BinaryHeap::new())),
        }
    }
}

impl Sessions {

    pub fn get(&self, key: &SessionKey) -> Option<dashmap::mapref::one::Ref<SessionKey, Session>> {
        self.map.get(key)
    }

    pub fn get_with_touch(&self, key: &SessionKey) -> Option<dashmap::mapref::one::Ref<SessionKey, Session>> {
        if let Some(mut session) = self.map.get_mut(key) {
            session.touch_last_active();
        };

        self.map.get(key)
    }

    pub fn get_mut_with_touch(&self, key: &SessionKey) -> Option<dashmap::mapref::one::RefMut<SessionKey, Session>> {
        if let Some(mut session) = self.map.get_mut(key) {
            session.touch_last_active();

            let mut queue = self.eviction_queue.lock().unwrap();
            queue.push(Reverse(EvictionEntry {
                last_active: session.last_active,
                last_active_seq: session.last_active_seq,
                key: *session.key()
            }));

            Some(session)
        } else {
            None
        }
    }

    pub fn remove(&self, key: &SessionKey) -> Option<Session> {
        self.map.remove(&key).map(|(_, session)| session)
    }

    pub fn insert(&self, key: SessionKey, session: Session) {
        self.map.insert(key, session);
    }

    pub fn keys(&self) -> Vec<SessionKey> {
        self.map.iter()
            .map(|entry| *entry.key())
            .collect()
    }

}
