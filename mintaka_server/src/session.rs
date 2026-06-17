use crate::app_state::{AppError, MemoryPermit, WorkerPermit};
use crate::stream_response_sender::StreamSessionResponseSender;
use dashmap::DashMap;
use mintaka::config::{Config, SearchObjective};
use mintaka::game_agent::{GameAgent, GameAgentData};
use mintaka::game_state::GameState;
use mintaka::memo::transposition_table::TTImportError;
use mintaka::protocol::command::Command;
use mintaka::protocol::response::Response;
use mintaka::protocol::results::{BestMove, CommandResult};
use mintaka::protocol::timer::Timer;
use rusty_renju::hash_key::HashKey;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::utils::byte_size::ByteSize;
use serde::{ser, Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::broadcast;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionToken(Uuid);

impl Display for SessionToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for SessionToken {
    type Err = AppError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::from_str(source).map_err(|_| AppError::Unauthorized)?))
    }
}

impl SessionToken {
    pub fn new_random() -> Self {
        Self(Uuid::new_v4())
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
    pub token: SessionToken,
    timer: Timer,
    state: AgentState,
    pub response_sender: SessionResponseSender,
    best_move: Option<BestMove>,
    abort_handle: Arc<AtomicBool>,
    time_to_hibernate: Option<Duration>,
    time_to_live: Option<Duration>,
    pub last_active: Instant,
    pub last_active_seq: u32,
    #[allow(dead_code)]
    memory_permit: MemoryPermit,
}

#[derive(Serialize, Deserialize)]
pub struct SessionData {
    pub config: Config,
    pub timer: Timer,
    pub agent: GameAgentData<{ RuleKind::Renju }>,
    pub best_move: Option<BestMove>,
    pub time_to_hibernate: Option<Duration>,
    pub time_to_live: Option<Duration>,
}

impl Session {
    pub fn new(
        config: Config,
        token: SessionToken,
        game_state: GameState<{ RuleKind::Renju }>,
        time_to_hibernate: Option<Duration>, time_to_live: Option<Duration>,
        memory_permit: MemoryPermit,
        response_sender: SessionResponseSender,
    ) -> Self {
        Self {
            config,
            token,
            timer: config.initial_timer,
            state: AgentState::Agent(GameAgent::from_state(config, game_state)),
            response_sender,
            best_move: None,
            abort_handle: Arc::new(AtomicBool::new(false)),
            time_to_hibernate,
            time_to_live,
            last_active: Instant::now(),
            last_active_seq: 0,
            memory_permit,
        }
    }

    pub fn from_data(
        data: SessionData,
        token: SessionToken,
        memory_permit: MemoryPermit,
        response_sender: SessionResponseSender,
    ) -> Result<Self, TTImportError> {
        let agent = data.agent.try_into()?;

        Ok(Self {
            config: data.config,
            token,
            timer: data.timer,
            state: AgentState::Agent(agent),
            response_sender,
            memory_permit,
            best_move: data.best_move,
            abort_handle: Arc::new(AtomicBool::new(false)),
            time_to_hibernate: data.time_to_hibernate,
            time_to_live: data.time_to_live,
            last_active: Instant::now(),
            last_active_seq: 0,
        })
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
        if !matches!(self.state, AgentState::Agent(_)) {
            return Err(AppError::SessionInComputing);
        }

        let AgentState::Agent(mut game_agent)
            = std::mem::replace(&mut self.state, AgentState::Permit(worker_permit))
                else { unreachable!("agent state checked before launch") };

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
            Some(time_to_live) => now.saturating_duration_since(self.last_active) >= time_to_live,
            None => false,
        }
    }

    pub fn should_hibernate(&self, now: Instant) -> bool {
        match self.time_to_hibernate {
            Some(time_to_hibernate) => now.saturating_duration_since(self.last_active) >= time_to_hibernate,
            None => false,
        }
    }

    pub fn live_until_epoch_secs(&self) -> Option<u64> {
        let time_to_live = self.time_to_live?;

        let live_time = SystemTime::now()
            .checked_sub(Instant::now().saturating_duration_since(self.last_active))?
            .checked_add(time_to_live)?;

        Some(live_time.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs())
    }

}

impl Drop for Session {
    fn drop(&mut self) {
        if let AgentState::Permit(_) = &self.state {
            self.abort_handle.store(true, Ordering::Relaxed);
        }
    }
}

impl<'a> TryFrom<&'a Session> for SessionData {
    type Error = AppError;

    fn try_from(session: &'a Session) -> Result<Self, Self::Error> {
        let AgentState::Agent(agent) = &session.state else {
            return Err(AppError::SessionInComputing);
        };

        Ok(Self {
            config: session.config,
            timer: session.timer,
            agent: GameAgentData::from(agent),
            best_move: session.best_move,
            time_to_hibernate: session.time_to_hibernate,
            time_to_live: session.time_to_live,
        })
    }
}

impl Serialize for Session {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        SessionData::try_from(self)
            .map_err(ser::Error::custom)?
            .serialize(serializer)
    }
}

struct EvictionEntry {
    last_active: Instant,
    last_active_seq: u32,
    key: SessionKey,
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
    map: DashMap<SessionKey, Session>,
    eviction_queue: Mutex<BinaryHeap<Reverse<EvictionEntry>>>,
}

impl Default for Sessions {
    fn default() -> Self {
        Self {
            map: DashMap::new(),
            eviction_queue: Mutex::new(BinaryHeap::new()),
        }
    }
}

impl Sessions {
    fn push_eviction_entry(&self, entry: EvictionEntry) {
        let mut queue = self.eviction_queue.lock().unwrap();
        queue.push(Reverse(entry));
    }

    fn push_eviction_entries(&self, entries: Vec<Reverse<EvictionEntry>>) {
        let mut queue = self.eviction_queue.lock().unwrap();
        queue.extend(entries);
    }

    fn pop_eviction_entry(&self) -> Option<Reverse<EvictionEntry>> {
        self.eviction_queue.lock().unwrap().pop()
    }

    fn eviction_entry_from_session(key: SessionKey, session: &Session) -> EvictionEntry {
        EvictionEntry {
            key,
            last_active: session.last_active,
            last_active_seq: session.last_active_seq,
        }
    }

    pub fn with<R>(&self, key: &SessionKey, f: impl FnOnce(&Session) -> R) -> Option<R> {
        self.map.get(key)
            .map(|session| f(&session))
    }

    pub fn with_touch<R>(&self, key: &SessionKey, f: impl FnOnce(&Session) -> R) -> Option<R> {
        let (result, entry) = {
            let mut session = self.map.get_mut(key)?;
            session.touch_last_active();

            let entry = Self::eviction_entry_from_session(*key, &session);
            let result = f(&session);

            (result, entry)
        };

        self.push_eviction_entry(entry);

        Some(result)
    }

    pub fn with_mut_touch<R>(&self, key: &SessionKey, f: impl FnOnce(&mut Session) -> R) -> Option<R> {
        let (result, entry) = {
            let mut session = self.map.get_mut(key)?;
            session.touch_last_active();

            let entry = Self::eviction_entry_from_session(*key, &session);
            let result = f(&mut session);

            (result, entry)
        };

        self.push_eviction_entry(entry);

        Some(result)
    }

    pub fn remove_lru_idle_until_memory(&self, memory_size: ByteSize) -> Vec<(SessionKey, Session)> {
        enum Candidate {
            Idle,
            Launched,
            Stale,
        }

        let mut removed_sessions = vec![];
        let mut deferred_entries = vec![];
        let mut acquired_memory = ByteSize::ZERO;

        while acquired_memory < memory_size {
            let Some(Reverse(entry)) = self.pop_eviction_entry() else {
                break;
            };

            match self.with(&entry.key, |session| {
                if session.last_active_seq != entry.last_active_seq {
                    Candidate::Stale
                } else if session.status() == SessionStatus::Idle {
                    Candidate::Idle
                } else {
                    Candidate::Launched
                }
            }).unwrap_or(Candidate::Stale) {
                Candidate::Idle => {
                    if let Ok(session) = self.remove_idle(&entry.key, Some(entry.last_active_seq)) {
                        acquired_memory += session.config.tt_size;
                        removed_sessions.push((entry.key, session));
                    }
                }
                Candidate::Launched => deferred_entries.push(Reverse(entry)),
                Candidate::Stale => {}
            }
        }

        if !deferred_entries.is_empty() {
            self.push_eviction_entries(deferred_entries);
        }

        removed_sessions
    }

    pub fn idle_keys_by_expiration(&self, now: Instant) -> (Vec<SessionKey>, Vec<SessionKey>) {
        let mut hibernation_keys = vec![];
        let mut expired_keys = vec![];

        for session in self.map.iter()
            .filter(|session| session.status() == SessionStatus::Idle)
        {
            if session.is_expired(now) {
                expired_keys.push(*session.key());
            } else if session.should_hibernate(now) {
                hibernation_keys.push(*session.key());
            }
        }

        (hibernation_keys, expired_keys)
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
        self.push_eviction_entry(EvictionEntry {
            key,
            last_active,
            last_active_seq,
        });
    }

    pub fn try_insert(&self, key: SessionKey, session: Session) -> Result<(), Session> {
        let last_active = session.last_active;
        let last_active_seq = session.last_active_seq;

        match self.map.entry(key) {
            dashmap::mapref::entry::Entry::Occupied(_) => return Err(session),
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                entry.insert(session);
            }
        }

        self.push_eviction_entry(EvictionEntry {
            key,
            last_active,
            last_active_seq,
        });

        Ok(())
    }

    pub fn keys(&self) -> Vec<SessionKey> {
        self.map.iter()
            .map(|entry| *entry.key())
            .collect()
    }
}
