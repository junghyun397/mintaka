use crate::app_state::{AppError, MemoryPermit, WorkerPermit};
use crate::stream_response_sender::StreamSessionResponseSender;
use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use dashmap::DashMap;
use mintaka::config::{Config, SearchObjective};
use mintaka::game_agent::GameAgent;
use mintaka::protocol::command::Command;
use mintaka::protocol::results::{BestMove, CommandResult};
use mintaka::protocol::response::Response;
use mintaka::game_state::GameState;
use rusty_renju::hash_key::HashKey;
use serde::ser::SerializeStruct;
use serde::{ser, Deserialize, Serialize, Serializer};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;
use uuid::Uuid;
use mintaka::protocol::timer::Timer;
use rusty_renju::notation::rule::RuleKind;

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

impl From<[u8; 16]> for SessionKey {
    fn from(bytes: [u8; 16]) -> Self {
        Self(Uuid::from_bytes(bytes))
    }
}

impl From<SessionKey> for [u8; 16] {
    fn from(key: SessionKey) -> Self {
        *key.0.as_bytes()
    }
}

impl SessionKey {
    pub fn new_random() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionToken(String);

impl Display for SessionToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for SessionToken {
    type Err = AppError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        if source.is_empty() {
            Err(AppError::Unauthorized)
        } else {
            Ok(Self(source.to_string()))
        }
    }
}

#[derive(Clone)]
pub struct SessionTokenCipher {
    cipher: Aes256Gcm,
}

impl SessionTokenCipher {
    const NONCE_LEN: usize = 12;

    pub fn new(secret: &str) -> Self {
        let digest = ring::digest::digest(&ring::digest::SHA256, secret.as_bytes());
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(digest.as_ref()));

        Self { cipher }
    }

    pub fn seal_session_key(&self, session_key: SessionKey) -> Result<SessionToken, AppError> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let session_key = Into::<[u8; 16]>::into(session_key);
        let ciphertext = self.cipher.encrypt(&nonce, session_key.as_slice()).unwrap();

        let mut token = Vec::with_capacity(Self::NONCE_LEN + ciphertext.len());
        token.extend_from_slice(&nonce);
        token.extend_from_slice(&ciphertext);

        Ok(SessionToken(URL_SAFE_NO_PAD.encode(token)))
    }

    pub fn open_session_token(&self, session_token: &SessionToken) -> Result<SessionKey, AppError> {
        let token = URL_SAFE_NO_PAD.decode(session_token.0.as_str())
            .map_err(|_| AppError::Unauthorized)?;

        let (nonce, text) = token.split_at_checked(Self::NONCE_LEN)
            .ok_or(AppError::Unauthorized)?;

        let session_key = self.cipher.decrypt(Nonce::from_slice(nonce), text)
            .map_err(|_| AppError::Unauthorized)?;

        let session_key = <[u8; 16]>::try_from(session_key.as_slice())
            .map_err(|_| AppError::Unauthorized)?;

        Ok(SessionKey::from(session_key))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum SessionStatus {
    Idle,
    InComputing,
    Hibernating
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionResponse {
    Response(Response),
    BestMove(BestMove),
}

pub struct SessionResultResponse {
    pub game_agent: GameAgent<{ RuleKind::Renju }>,
    pub best_move: BestMove,
}

impl Debug for SessionResultResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.game_agent.state.history, self.best_move.best_move)
    }
}

pub enum AgentState {
    Agent(GameAgent<{ RuleKind::Renju }>),
    Permit(WorkerPermit)
}

pub type SessionResponseSender = broadcast::Sender<SessionResponse>;
pub type SessionResponseReceiver = broadcast::Receiver<SessionResponse>;

pub struct Session {
    pub config: Config,
    timer: Timer,
    state: AgentState,
    pub response_sender: SessionResponseSender,
    #[allow(dead_code)]
    memory_permit: MemoryPermit,
    best_move: Option<BestMove>,
    abort_handle: Arc<AtomicBool>,
    time_to_live: Option<Duration>,
    pub last_active: Instant,
    pub last_active_seq: u32,
}

#[derive(Deserialize)]
pub struct SessionData {
    pub config: Config,
    pub timer: Timer,
    pub agent: GameAgent<{ RuleKind::Renju }>,
    pub best_move: Option<BestMove>,
    pub time_to_live: Option<Duration>,
}

impl Session {
    pub fn new(config: Config, game_state: GameState<{ RuleKind::Renju }>, time_to_live: Option<Duration>, memory_permit: MemoryPermit, response_sender: SessionResponseSender) -> Self {
        Self {
            config,
            timer: config.initial_timer,
            state: AgentState::Agent(GameAgent::from_state(config, game_state)),
            response_sender,
            memory_permit,
            best_move: None,
            abort_handle: Arc::new(AtomicBool::new(false)),
            time_to_live,
            last_active: Instant::now(),
            last_active_seq: 0,
        }
    }

    pub fn from_data(data: SessionData, memory_permit: MemoryPermit, response_sender: SessionResponseSender) -> Self {
        Self {
            config: data.config,
            timer: data.timer,
            state: AgentState::Agent(data.agent),
            response_sender,
            memory_permit,
            best_move: data.best_move,
            abort_handle: Arc::new(AtomicBool::new(false)),
            time_to_live: data.time_to_live,
            last_active: Instant::now(),
            last_active_seq: 0,
        }
    }

    pub fn game_agent(&self) -> Result<&GameAgent<{ RuleKind::Renju }>, AppError> {
        match &self.state {
            AgentState::Agent(agent) => Ok(agent),
            AgentState::Permit { .. } => Err(AppError::SessionInComputing),
        }
    }

    pub fn game_agent_mut(&mut self) -> Result<&mut GameAgent<{ RuleKind::Renju }>, AppError> {
        match &mut self.state {
            AgentState::Agent(agent) => Ok(agent),
            AgentState::Permit { .. } => Err(AppError::SessionInComputing),
        }
    }

    pub fn touch_last_active(&mut self) {
        self.last_active = Instant::now();
        self.last_active_seq += 1;
    }

    pub fn command(&mut self, command: Command) -> Result<CommandResult, AppError> {
        self.game_agent_mut()?.command(command)
            .map_err(AppError::GameError)
    }

    pub fn launch(
        &mut self,
        response_sender: StreamSessionResponseSender,
        result_sender: tokio::sync::oneshot::Sender<SessionResultResponse>,
        worker_permit: WorkerPermit,
        _nodes_polling_interval_ms: Option<u32>,
    ) -> Result<(), AppError> {
        let AgentState::Agent(mut game_agent)
            = std::mem::replace(&mut self.state, AgentState::Permit(worker_permit))
                else { return Err(AppError::SessionInComputing) };

        self.abort_handle.store(false, Ordering::Relaxed);
        let abort_flag = self.abort_handle.clone();

        let config = self.config;
        let timer = self.timer;

        tokio::task::spawn_blocking(move || {
            let best_move = game_agent.launch::<Instant>(
                config,
                timer,
                SearchObjective::Best,
                response_sender,
                Arc::new(AtomicU32::new(0)),
                abort_flag
            );

            let _ = result_sender.send(SessionResultResponse { game_agent, best_move });
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

    pub fn restore(&mut self, game_agent: GameAgent<{ RuleKind::Renju }>) -> Result<(), AppError> {
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

    pub fn live_until_epoch_secs(&self) -> Option<u64> {
        let time_to_live = self.time_to_live?;
        let live_time = SystemTime::now()
            - Instant::now().duration_since(self.last_active)
            + time_to_live;

        Some(live_time.duration_since(UNIX_EPOCH).unwrap().as_secs())
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
    fn push_eviction_entry(&self, key: SessionKey, last_active: Instant, last_active_seq: u32) {
        let mut queue = self.eviction_queue.lock().unwrap();
        queue.push(Reverse(EvictionEntry {
            last_active,
            last_active_seq,
            key,
        }));
    }

    pub fn get(&self, key: &SessionKey) -> Option<dashmap::mapref::one::Ref<'_, SessionKey, Session>> {
        self.map.get(key)
    }

    pub fn get_with_touch(&self, key: &SessionKey) -> Option<dashmap::mapref::one::Ref<'_, SessionKey, Session>> {
        if let Some(mut session) = self.map.get_mut(key) {
            session.touch_last_active();

            self.push_eviction_entry(*session.key(), session.last_active, session.last_active_seq);
        };

        self.map.get(key)
    }

    pub fn get_mut_with_touch(&self, key: &SessionKey) -> Option<dashmap::mapref::one::RefMut<'_, SessionKey, Session>> {
        if let Some(mut session) = self.map.get_mut(key) {
            session.touch_last_active();

            self.push_eviction_entry(*session.key(), session.last_active, session.last_active_seq);

            Some(session)
        } else {
            None
        }
    }

    pub fn remove_idle(&self, key: &SessionKey, last_active_seq: Option<u32>) -> Result<Session, AppError> {
        if let Some((_, session)) = self.map.remove_if(key, |_, session|
            session.status() == SessionStatus::Idle
                && last_active_seq.map_or(true, |seq| session.last_active_seq == seq)
        ) {
            return Ok(session);
        }

        match self.map.get(key) {
            Some(_) => Err(AppError::SessionInComputing),
            None => Err(AppError::SessionNotFound),
        }
    }

    pub fn insert(&self, key: SessionKey, session: Session) {
        let last_active = session.last_active;
        let last_active_seq = session.last_active_seq;

        self.map.insert(key, session);
        self.push_eviction_entry(key, last_active, last_active_seq);
    }

    pub fn try_insert(&self, key: SessionKey, session: Session) -> Result<(), Session> {
        let last_active = session.last_active;
        let last_active_seq = session.last_active_seq;

        match self.map.entry(key) {
            dashmap::mapref::entry::Entry::Occupied(_) => Err(session),
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                entry.insert(session);
                self.push_eviction_entry(key, last_active, last_active_seq);

                Ok(())
            }
        }
    }

    pub fn keys(&self) -> Vec<SessionKey> {
        self.map.iter()
            .map(|entry| *entry.key())
            .collect()
    }
}
