use crate::config::{Config, SearchObjective};
use crate::eval::evaluator::{ActiveEvaluator, Evaluator};
use crate::eval::heuristic_evaluator::HeuristicEvaluator;
use crate::game_state::GameState;
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TranspositionTable;
use crate::principal_variation::PrincipalVariation;
use crate::protocol::command::Command;
use crate::protocol::game_result::GameResult;
use crate::protocol::response::{Response, ResponseSender};
use crate::search::iterative_deepening;
use crate::thread_data::ThreadData;
use crate::thread_type::{MainThread, WorkerThread};
use crate::time_manager::TimeManager;
use rusty_renju::bitfield::Bitfield;
use rusty_renju::memo::abstract_transposition_table::AbstractTranspositionTable;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::score::Score;
use rusty_renju::utils::byte_size::ByteSize;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct BestMove {
    pub hash: HashKey,
    pub pos: MaybePos,
    pub score: Score,
    pub selective_depth: usize,
    pub total_nodes_in_1k: usize,
    pub time_elapsed: Duration,
    pub pv: PrincipalVariation,
}

#[derive(Debug)]
pub enum GameError {
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ComputingResource {
    pub workers: u32,
    pub tt_size: ByteSize,
    pub time: Option<Duration>,
    pub nodes_in_1k: Option<usize>,
}

pub struct GameAgent {
    pub config: Config,
    pub state: GameState,
    pub evaluator: ActiveEvaluator,
    tt: TranspositionTable,
    ht: HistoryTable,
    executed_moves: Bitfield,
    pub time_manager: TimeManager,
}

impl GameAgent {

    pub fn new(config: Config) -> Self {
        Self::from_state(config, GameState::default())
    }

    pub fn from_state(config: Config, state: GameState) -> Self {
        let tt = TranspositionTable::new_with_size(config.tt_size);

        Self {
            config,
            state,
            evaluator: ActiveEvaluator::from_state(&state),
            tt,
            ht: HistoryTable::EMPTY,
            executed_moves: Bitfield::default(),
            time_manager: config.initial_timer.into(),
        }
    }

    pub fn command(&mut self, command: Command) -> Result<Option<GameResult>, GameError> {
        match command {
            Command::Play(action) => {
                match action {
                    MaybePos::NONE => self.state.pass_mut(),
                    pos => {
                        let pos = pos.unwrap();

                        if !self.state.board.is_pos_empty(pos) {
                            return Err(GameError::StoneAlreadyExist);
                        }

                        if !self.state.board.is_legal_move(pos) {
                            return Err(GameError::ForbiddenMove);
                        }

                        self.state.set_mut(pos);
                        self.evaluator.update(&self.state);

                        if let Some(winner) = self.state.board.find_winner(pos) {
                            return Ok(Some(GameResult::Win(winner)));
                        }
                    }
                }

                if self.state.board.stones == pos::U8_BOARD_SIZE {
                    return Ok(Some(GameResult::Full));
                } else if self.state.len() >= self.config.draw_condition {
                    return Ok(Some(GameResult::Draw));
                }
            },
            Command::Undo => {
                match self.state.history.pop_mut() {
                    None => return Err(GameError::NoHistoryToUndo),
                    Some(action) => {
                        match action {
                            MaybePos::NONE => {
                                self.state.board.switch_player_mut();
                            },
                            pos => {
                                self.state.board.unset_mut(pos.unwrap());
                            }
                        }

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
            Command::Set { pos, color } => {
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
            Command::Unset { pos, color } => {
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
                self.reinit_from_state(GameState::default());
            },
            Command::Load(boxed) => {
                self.reinit_from_state(GameState::from_board_and_history(boxed.0, boxed.1));
            },
            Command::TurnTime(time) => {
                self.time_manager.timer.turn = time;
            },
            Command::IncrementTime(time) => {
                self.time_manager.timer.increment = time;
            },
            Command::TotalTime(time) => {
                self.time_manager.timer.total_remaining = time;
            },
            Command::ConsumeTime(time) => {
                self.time_manager.timer.consume(time);
            },
            Command::Pondering(enable) => {
                self.config.pondering = enable;
            }
            Command::MaxNodes { in_1k } => {
                self.config.max_nodes_in_1k = Some(in_1k);
            },
            Command::Workers(workers) => {
                self.config.workers = workers;
            },
            Command::MaxMemory(size) => {
                const HEAP_MEMORY_MARGIN_IN_KIB: ByteSize = ByteSize::from_mib(10);

                self.tt.resize(size - HEAP_MEMORY_MARGIN_IN_KIB);
            },
            Command::Rule(kind) => {
                self.config.rule_kind = kind;
            },
        };

        Ok(None)
    }

    pub fn commands(&mut self, commands: Vec<Command>) -> Result<Option<GameResult>, GameError> {
        let mut result = None;

        for command in commands.into_iter() {
            let local_result = self.command(command)?;

            result = result.or(local_result);
        }

        Ok(result)
    }

    fn next_computing_resource(&self) -> ComputingResource {
        ComputingResource {
            workers: self.config.workers,
            tt_size: self.config.tt_size,
            time: self.time_manager.next_running_time(),
            nodes_in_1k: self.config.max_nodes_in_1k,
        }
    }

    pub fn launch(&mut self, search_objective: SearchObjective, response_sender: impl ResponseSender, aborted: Arc<AtomicBool>) -> BestMove {
        let computing_resource = self.next_computing_resource();

        let started_time = std::time::Instant::now();

        aborted.store(false, Ordering::Relaxed);
        let global_counter_in_1k = Arc::new(AtomicUsize::new(0));

        response_sender.response(Response::Begins(computing_resource));

        let tt_view = self.tt.view();

        let (main_td, score, best_move) = std::thread::scope(|s| {
            let state = self.state;

            let mut main_td = ThreadData::new(
                MainThread::new(
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

            let handle = s.spawn(move || {
                let (score, best_move) = iterative_deepening::<{ RuleKind::Renju }, MainThread<_>>(
                    &mut main_td, state
                );

                (main_td, score, best_move)
            });

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

                s.spawn(move || {
                    iterative_deepening::<{ RuleKind::Renju }, WorkerThread>(
                        &mut worker_td, state
                    );
                });
            }

            handle.join().unwrap()
        });

        self.tt.increase_age();
        self.ht.increase_age();
        self.ht = *main_td.ht;

        self.executed_moves.set_idx(self.state.len());

        let time_elapsed = started_time.elapsed();

        self.time_manager.timer.consume(time_elapsed);
        self.time_manager.timer.apply_increment();
        self.time_manager.time_history.push((best_move, time_elapsed));

        BestMove {
            hash: self.state.board.hash_key,
            pos: best_move,
            score,
            selective_depth: main_td.selective_depth,
            total_nodes_in_1k: main_td.batch_counter.count_global_in_1k(),
            time_elapsed,
            pv: main_td.root_pv
        }
    }

    fn reinit_from_state(&mut self, state: GameState) {
        self.state = state;

        self.evaluator = HeuristicEvaluator::from_state(&self.state);

        self.tt.clear(self.config.workers);
        self.executed_moves = Bitfield::default();

        self.time_manager = TimeManager::from(self.config.initial_timer);
    }

}

impl Serialize for GameAgent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
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

impl<'de> Deserialize<'de> for GameAgent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        #[derive(Deserialize)]
        struct GameAgentData {
            config: Config,
            state: GameState,
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
