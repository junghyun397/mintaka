use crate::config::{Config, SearchObjective};
use crate::eval::evaluator::{ActiveEvaluator, Evaluator};
use crate::eval::heuristic_evaluator::HeuristicEvaluator;
use crate::game_state::{GameState, GameStateData};
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TranspositionTable;
use crate::protocol::command::Command;
pub use crate::protocol::response::{ComputingResource, Response, ResponseSender};
use crate::protocol::results::{BestMove, CommandResult, GameResult};
use crate::search::iterative_deepening;
use crate::thread_data::ThreadData;
use crate::thread_type::{MainThread, WorkerThread};
use crate::time::{TimeManager, Timer};
use crate::utils::time::MonotonicClock;
use rusty_renju::bitfield::Bitfield;
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::utils::byte_size::ByteSize;
use rusty_renju::utils::empty::Empty;
#[cfg(feature = "serde")]
use serde::ser::SerializeStruct;
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

#[cfg(not(feature = "rayon"))]
macro_rules! search_scope {
    ($body:expr) => {
        std::thread::scope($body)
    };
}

#[cfg(feature = "rayon")]
macro_rules! search_scope {
    ($body:expr) => {
        rayon::in_place_scope($body)
    };
}

#[cfg(not(feature = "rayon"))]
fn spawn_search_worker<'scope, 'env, F>(scope: &'scope std::thread::Scope<'scope, 'env>, worker: F)
where F: FnOnce() + Send + 'scope {
    scope.spawn(worker);
}

#[cfg(feature = "rayon")]
fn spawn_search_worker<'scope, F>(scope: &rayon::Scope<'scope>, worker: F)
where F: FnOnce() + Send + 'scope {
    scope.spawn(move |_| worker());
}

#[derive(Debug)]
pub enum GameError {
    InvalidConfig,
    HashMismatch,
    StoneAlreadyExist,
    StoneDoesNotExist,
    StoneColorMismatch,
    ForbiddenMove,
    NoHistoryToUndo,
    NoTimeManagement,
}

impl Display for GameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GameError::InvalidConfig => write!(f, "invalid config"),
            GameError::HashMismatch => write!(f, "hash mismatch"),
            GameError::StoneAlreadyExist => write!(f, "stone already exist"),
            GameError::StoneDoesNotExist => write!(f, "stone does not exist"),
            GameError::StoneColorMismatch => write!(f, "stone color mismatch"),
            GameError::ForbiddenMove => write!(f, "forbidden move"),
            GameError::NoHistoryToUndo => write!(f, "no history to undo"),
            GameError::NoTimeManagement => write!(f, "no time management"),
        }
    }
}

impl std::error::Error for GameError {}

pub struct GameAgent<const R: RuleKind> {
    pub config: Config,
    pub state: GameState<R>,
    pub evaluator: ActiveEvaluator<R>,
    tt: TranspositionTable,
    ht: HistoryTable,
    executed_moves: Bitfield,
    pub time_manager: TimeManager,
}

impl<const R: RuleKind> GameAgent<R> {
    pub fn new(config: Config) -> Self {
        Self::from_state(config, GameState::empty())
    }

    pub fn from_state(config: Config, state: GameState<R>) -> Self {
        let tt = TranspositionTable::new_with_size(config.tt_size);

        Self {
            config,
            state,
            evaluator: ActiveEvaluator::from_state(&state),
            tt,
            ht: HistoryTable::empty(),
            executed_moves: Bitfield::empty(),
            time_manager: TimeManager::new(config.initial_timer, config.dynamic_time),
        }
    }

    pub fn command(&mut self, command: Command) -> Result<CommandResult, GameError> {
        match command {
            Command::Play { hash, pos } => {
                if hash != self.state.board.hash_key {
                    return Err(GameError::HashMismatch);
                }
                
                if let Some(pos) = pos.ok() {
                    if !self.state.board.is_pos_empty(pos) {
                        return Err(GameError::StoneAlreadyExist);
                    }

                    if !self.state.board.is_legal_move(pos) {
                        return Err(GameError::ForbiddenMove);
                    }

                    let artifact = self.state.play_mut(pos);
                    self.evaluator.play(&self.state.board, artifact, pos.into());

                    if let Some(winner) = self.state.board.find_winner(pos) {
                        return Ok(CommandResult::finished(self.state.board.hash_key, GameResult::Win(winner)));
                    }
                } else {
                    self.state.pass_mut();
                }

                if self.state.board.stones == pos::U8_BOARD_SIZE {
                    return Ok(CommandResult::finished(self.state.board.hash_key, GameResult::Full));
                } else if self.state.len() >= self.config.draw_condition as usize {
                    return Ok(CommandResult::finished(self.state.board.hash_key, GameResult::Draw));
                }
            },
            Command::Undo { hash } => {
                if hash != self.state.board.hash_key {
                    return Err(GameError::HashMismatch);
                }

                match self.state.history.last_action() {
                    None => return Err(GameError::NoHistoryToUndo),
                    Some(action) => {
                        let artifact = self.state.undo_rebuild_mut();

                        self.evaluator.undo(&self.state.board, artifact, action);

                        if self.executed_moves.is_hot_idx(self.state.len()) {
                            self.executed_moves.unset_idx(self.state.len());

                            if let Some((pos, time)) = self.time_manager.time_history.pop()
                                && pos == action
                            {
                                self.time_manager.timer.append(time);
                            }
                        }
                    }
                }
            },
            Command::Set { hash, pos, color } => {
                if hash != self.state.board.hash_key {
                    return Err(GameError::HashMismatch);
                }

                if !self.state.board.is_pos_empty(pos) {
                    return Err(GameError::StoneAlreadyExist);
                }

                self.state.movegen_window.imprint_window(pos);

                if self.state.board.player_color == color {
                    self.state.board.set_mut(pos);
                    self.state.board.switch_player_mut();
                } else {
                    self.state.board.switch_player_mut();
                    self.state.board.set_mut(pos);
                }
            },
            Command::Unset { hash, pos, color } => {
                if hash != self.state.board.hash_key {
                    return Err(GameError::HashMismatch);
                }

                match self.state.board.stone_kind(pos) {
                    Some(stone_color) if stone_color == color => {
                        if self.state.board.player_color == color {
                            self.state.board.switch_player_mut();
                            self.state.board.unset_mut(pos);
                        } else {
                            self.state.board.unset_mut(pos);
                            self.state.board.switch_player_mut();
                        }

                        if let Some((index, time)) = self.time_manager.time_history.iter()
                            .enumerate()
                            .find_map(|(index, &(action, time))|
                                (action == pos.into()).then_some((index, time))
                            )
                        {
                            self.time_manager.time_history.remove(index);
                            self.time_manager.timer.append(time);
                        }

                    },
                    Some(_) => return Err(GameError::StoneColorMismatch),
                    None => return Err(GameError::StoneDoesNotExist),
                }
            },
            Command::BatchSet { player_moves, opponent_moves } => {
                let (black_stones, white_stones) =
                    match self.state.board.player_color {
                        Color::Black => (player_moves, opponent_moves),
                        Color::White => (opponent_moves, player_moves),
                    };

                let batch_color =
                    Color::player_color_from_each_moves(black_stones.len(), white_stones.len());

                let player = if batch_color == Color::Black {
                    self.state.board.player_color
                } else {
                    !self.state.board.player_color
                };

                self.state.board.batch_set_each_color_mut(
                    black_stones.into_boxed_slice(),
                    white_stones.into_boxed_slice(),
                    player
                );

                self.evaluator = HeuristicEvaluator::from_state(&self.state);
            },
            Command::Clear => {
                self.reinit_from_state(GameStateData {
                    board_data: (&Board::<R>::empty()).into(),
                    history: History::empty(),
                });
            },
            Command::Init(state) => {
                self.reinit_from_state(*state);
            },
            Command::Sync(state) => {
                self.sync_state(*state);
            }
            Command::TurnTime(time) => {
                self.time_manager.timer.turn = Some(time);
            },
            Command::IncrementTime(time) => {
                self.time_manager.timer.increment = time;
            },
            Command::TotalTime(time) => {
                self.time_manager.timer.total_remaining = Some(time);
            },
            Command::ConsumeTime(time) => {
                self.time_manager.timer.consume(time);
            },
            Command::Pondering(enable) => {
                self.config.pondering = enable;
            },
            Command::MaxNodes { in_1k } => {
                self.config.max_nodes_in_1k = Some(in_1k);
            },
            Command::Workers(workers) => {
                self.config.workers = workers;
            },
            Command::MaxMemory(size) => {
                self.resize_tt(size);
            },
            Command::Config(config) => {
                let old_config = self.config;
                self.config = config;

                if old_config.tt_size != self.config.tt_size {
                    self.resize_tt(self.config.tt_size);
                }

                self.time_manager.timer = Timer {
                    turn: self.config.initial_timer.turn,
                    increment: self.config.initial_timer.increment,
                    total_remaining: self.time_manager.timer.total_remaining,
                }
            }
        };

        Ok(CommandResult::hash(self.state.board.hash_key))
    }

    pub fn batch_command(&mut self, commands: Vec<Command>) -> Result<(u32, CommandResult), (u32, GameError)> {
        let mut command_result = None;

        for (index, command) in commands.into_iter().enumerate() {
            match self.command(command) {
                Ok(CommandResult { hash_key, result: Some(result) }) =>
                    return Ok((index as u32, CommandResult::finished(hash_key, result))),
                Err(error) =>
                    return Err((index as u32, error)),
                Ok(result) => command_result = Some(result),
            }
        }

        Ok(command_result.map(|result| (0, result)).unwrap())
    }

    fn next_computing_resource(&self) -> ComputingResource {
        ComputingResource {
            workers: self.config.workers,
            tt_size: self.config.tt_size,
            time: self.time_manager.next_running_time(),
            nodes_in_1k: self.config.max_nodes_in_1k,
        }
    }

    pub fn launch<CLK: MonotonicClock>(
        &mut self,
        search_objective: SearchObjective,
        response_sender: impl ResponseSender,
        global_counter_in_1k: Arc<AtomicU32>,
        aborted: Arc<AtomicBool>,
    ) -> BestMove {
        let started_time = CLK::now();

        let computing_resource = self.next_computing_resource();

        global_counter_in_1k.store(0, Ordering::Relaxed);
        aborted.store(false, Ordering::Relaxed);

        response_sender.response(Response::Begins(computing_resource));

        if self.evaluator.hash_key() != self.state.board.hash_key {
            self.evaluator.init(&self.state.board)
        }

        let tt_view = self.tt.view();

        let (main_td, score, best_move) = search_scope!(|s| {
            let state = self.state;

            for tid in 1 .. self.config.workers {
                let mut worker_td = ThreadData::new(
                    WorkerThread, tid,
                    search_objective,
                    self.config,
                    self.evaluator.clone(),
                    tt_view,
                    self.ht,
                    &aborted, &global_counter_in_1k
                );

                spawn_search_worker(s, move || {
                    iterative_deepening::<CLK, R, WorkerThread>(
                        &mut worker_td, state
                    );
                });
            }

            let mut main_td = ThreadData::new(
                MainThread::<CLK, _>::new(
                    state.board.hash_key,
                    response_sender,
                    started_time,
                    computing_resource.time,
                ),
                0,
                search_objective,
                self.config,
                self.evaluator.clone(),
                tt_view,
                self.ht,
                &aborted, &global_counter_in_1k,
            );

            let (score, best_move) = iterative_deepening::<CLK, R, MainThread<_, _>>(
                &mut main_td, state
            );

            (main_td, score, best_move)
        });

        self.ht = *main_td.ht;

        self.tt.increase_age();
        self.ht.increase_age();

        self.executed_moves.set_idx(self.state.len());

        let time_elapsed = started_time.elapsed();

        self.time_manager.timer.consume(time_elapsed);
        self.time_manager.timer.apply_increment();
        self.time_manager.time_history.push((best_move, time_elapsed));

        BestMove {
            position_hash: self.state.board.hash_key,
            best_move,
            score,
            selective_depth: main_td.selective_depth as u32,
            total_nodes_in_1k: main_td.batch_counter.count_global_in_1k(),
            time_elapsed,
            pv: main_td.root_pv
        }
    }

    fn sync_state(&mut self, data: GameStateData) {
        self.state = data.into();
        self.evaluator = ActiveEvaluator::from_state(&self.state);
        self.executed_moves = Bitfield::empty();
    }

    fn reinit_from_state(&mut self, data: GameStateData) {
        self.state = data.into();

        self.evaluator = ActiveEvaluator::from_state(&self.state);

        self.tt.clear();
        self.ht = HistoryTable::empty();
        self.executed_moves = Bitfield::empty();
    }

    fn resize_tt(&mut self, size: ByteSize) {
        const HEAP_MEMORY_MARGIN: ByteSize = ByteSize::from_mib(10);

        self.tt.resize(size - HEAP_MEMORY_MARGIN);
    }
}

#[cfg(feature = "serde")]
impl<const R: RuleKind> Serialize for GameAgent<R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut state = serializer.serialize_struct("GameAgent", 6)?;
        state.serialize_field("config", &self.config)?;
        state.serialize_field("state", &self.state)?;
        state.serialize_field("tt", &self.tt)?;
        state.serialize_field("ht", &self.ht)?;
        state.serialize_field("time_management", &self.time_manager)?;
        state.serialize_field("executed_moves", &self.executed_moves)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, const R: RuleKind> Deserialize<'de> for GameAgent<R> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        #[derive(Deserialize)]
        struct GameAgentData<const R: RuleKind> {
            config: Config,
            state: GameState<R>,
            tt: TranspositionTable,
            ht: HistoryTable,
            time_manager: TimeManager,
            executed_moves: Bitfield,
        }

        let data = GameAgentData::deserialize(deserializer)?;

        let evaluator = ActiveEvaluator::from_state(&data.state);

        Ok(Self {
            config: data.config,
            state: data.state,
            evaluator,
            tt: data.tt,
            ht: data.ht,
            time_manager: data.time_manager,
            executed_moves: data.executed_moves,
        })
    }
}
