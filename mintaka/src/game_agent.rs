use crate::config::{Config, SearchObjective};
use crate::eval::evaluator::{ActiveEvaluator, Evaluator};
use crate::eval::heuristic_evaluator::HeuristicEvaluator;
use crate::game_state::{GameState, GameStateData};
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::{TTImportError, TranspositionTable};
use crate::protocol::command::Command;
pub use crate::protocol::response::{ComputingResource, Response, ResponseSender};
use crate::protocol::results::{BestMove, CommandResult, GameResult};
use crate::protocol::timer::Timer;
use crate::search::iterative_deepening;
use crate::thread_data::ThreadData;
use crate::thread_type::{MainThread, WorkerThread};
use crate::time_manager::TimeManager;
use crate::utils::monotonic_clock::MonotonicClock;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::utils::empty::Empty;
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

#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "GameAgentData<R>"))]
pub struct GameAgent<const R: RuleKind> {
    pub state: GameState<R>,
    pub evaluator: ActiveEvaluator<R>,
    tt: TranspositionTable,
    ht: HistoryTable,
}

impl<const R: RuleKind> GameAgent<R> {
    pub fn new(config: Config) -> Self {
        Self::from_state(config, GameState::empty())
    }

    pub fn from_state(config: Config, state: GameState<R>) -> Self {
        let tt = TranspositionTable::new_with_size(config.tt_size);

        Self {
            state,
            evaluator: ActiveEvaluator::from_state(&state),
            tt,
            ht: HistoryTable::empty(),
        }
    }

    fn reinit_from_state(&mut self, state: GameState<R>) {
        self.state = state;

        self.evaluator = ActiveEvaluator::from_state(&self.state);

        self.tt.clear();
        self.ht = HistoryTable::empty();
    }

    fn sync_state(&mut self, data: GameStateData) {
        self.state = data.into();
        self.evaluator = ActiveEvaluator::from_state(&self.state);
    }

    pub fn command(&mut self, command: Command) -> Result<CommandResult, GameError> {
        match command {
            Command::Play { hash, pos, draw_condition } => {
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
                } else if draw_condition.is_some_and(|draw_in| self.state.len() as u32 >= draw_in) {
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
                self.reinit_from_state(GameState::empty());
            },
            Command::Init(state) => {
                self.reinit_from_state((*state).into());
            },
            Command::Sync(state) => {
                self.sync_state(*state);
            }
            Command::RebuildTT(size)  => {
                self.tt.resize(size);
            },
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

    fn next_computing_resource<CLK: MonotonicClock>(&self, config: Config, timer: Timer, started_time: CLK) -> (ComputingResource, TimeManager<CLK>) {
        let time_manager = TimeManager::init(timer, started_time);

        let resource = ComputingResource {
            workers: config.workers,
            time_limit: time_manager.hard_limit(),
            nodes_in_1k: config.max_nodes_in_1k,
        };

        (resource, time_manager)
    }

    pub fn launch<CLK: MonotonicClock>(
        &mut self,
        config: Config,
        timer: Timer,
        search_objective: SearchObjective,
        response_sender: impl ResponseSender,
        global_counter_in_1k: Arc<AtomicU32>,
        aborted: Arc<AtomicBool>,
    ) -> BestMove {
        let started_time = CLK::now();

        let (computing_resource, time_manager) = self.next_computing_resource(config, timer, started_time);

        global_counter_in_1k.store(0, Ordering::Relaxed);
        aborted.store(false, Ordering::Relaxed);

        response_sender.response(Response::Begins(computing_resource));

        if self.evaluator.hash_key() != self.state.board.hash_key {
            self.evaluator.init(&self.state.board)
        }

        let tt_view = self.tt.view();

        let (main_td, score, best_move) = search_scope!(|s| {
            let state = self.state;

            for tid in 1 .. config.workers {
                let mut worker_td = ThreadData::new(
                    WorkerThread::new(), tid,
                    search_objective,
                    config,
                    self.evaluator.clone(),
                    tt_view,
                    self.ht,
                    &aborted, &global_counter_in_1k
                );

                spawn_search_worker(s, move || {
                    iterative_deepening::<R, WorkerThread<CLK>>(
                        &mut worker_td, state
                    );
                });
            }

            let mut main_td = ThreadData::new(
                MainThread::<CLK, _>::new(
                    response_sender,
                    time_manager,
                ),
                0,
                search_objective,
                config,
                self.evaluator.clone(),
                tt_view,
                self.ht,
                &aborted, &global_counter_in_1k,
            );

            let (score, best_move) = iterative_deepening::<R, MainThread<_, _>>(
                &mut main_td, state
            );

            (main_td, score, best_move)
        });

        self.ht = *main_td.ht;

        self.tt.increase_age();
        self.ht.increase_age();

        BestMove {
            position_hash: self.state.board.hash_key,
            best_move,
            score,
            selective_depth: main_td.selective_depth as u32,
            total_nodes_in_1k: main_td.batch_counter.count_global_in_1k(),
            time_elapsed: started_time.elapsed(),
            pv: main_td.root_pv
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GameAgentData<const R: RuleKind> {
    state: GameState<R>,
    tt: Vec<u8>,
    ht: HistoryTable,
}

impl<const R: RuleKind> From<&GameAgent<R>> for GameAgentData<R> {
    fn from(agent: &GameAgent<R>) -> Self {
        Self {
            state: agent.state,
            tt: agent.tt.export(9),
            ht: agent.ht,
        }
    }
}

impl<const R: RuleKind> TryFrom<GameAgentData<R>> for GameAgent<R> {
    type Error = TTImportError;

    fn try_from(data: GameAgentData<R>) -> Result<Self, Self::Error> {
        let tt = TranspositionTable::import(data.tt)?;
        let evaluator = ActiveEvaluator::from_state(&data.state);

        Ok(Self {
            state: data.state,
            evaluator,
            tt,
            ht: data.ht,
        })
    }
}

#[cfg(feature = "serde")]
impl<const R: RuleKind> serde::Serialize for GameAgent<R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        GameAgentData::from(self).serialize(serializer)
    }
}
