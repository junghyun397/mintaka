use crate::config::Config;
use crate::game_state::GameState;
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TranspositionTable;
use crate::protocol::command::Command;
use crate::protocol::message;
use crate::protocol::message::{Message, MessageSender};
use crate::protocol::response::{Response, ResponseSender};
use crate::search;
use crate::thread_data::ThreadData;
use crate::thread_type::{MainThread, WorkerThread};
use crate::utils::time_manager::TimeManager;
use rusty_renju::bitfield::Bitfield;
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::memo::abstract_transposition_table::AbstractTranspositionTable;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::value::Score;
use rusty_renju::utils::byte_size::ByteSize;
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
    pub total_nodes_in_1k: usize,
    pub time_elapsed: Duration,
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

#[derive(Default, Serialize, Deserialize)]
pub struct TimeManagement {
    time_manager: TimeManager,
    time_history: Vec<(MaybePos, Duration)>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ComputingResource {
    pub workers: u32,
    pub tt_size: ByteSize,
    pub time: Option<Duration>,
    pub nodes_in_1k: usize,
}

#[derive(Serialize, Deserialize)]
pub struct GameAgent {
    pub state: GameState,
    pub config: Config,
    pub time_management: Option<TimeManagement>,
    executed_moves: Bitfield,
    tt: TranspositionTable,
    ht: HistoryTable,
}

impl GameAgent {

    pub fn new(config: Config) -> Self {
        let tt = TranspositionTable::new_with_size(config.tt_size);

        Self {
            state: GameState::default(),
            config,
            executed_moves: Bitfield::default(),
            time_management: config.time_management.then(TimeManagement::default),
            tt,
            ht: HistoryTable {},
        }
    }

    pub fn from_state(config: Config, board: Board, history: History) -> Self {
        let state = GameState {
            board,
            history,
            movegen_window: (&board.hot_field).into(),
            move_scores: (&board.hot_field).into(),
        };

        let tt = TranspositionTable::new_with_size(config.tt_size);

        Self {
            state,
            config,
            executed_moves: Bitfield::default(),
            time_management: config.time_management.then(TimeManagement::default),
            tt,
            ht: HistoryTable {},
        }
    }

    pub fn command(&mut self, message_sender: &MessageSender, command: Command) -> Result<(), GameError> {
        match command {
            Command::Play(action) => {
                match action {
                    MaybePos::NONE => self.state.pass_mut(),
                    pos => {
                        if !self.state.board.is_pos_empty(pos.unwrap()) {
                            return Err(GameError::StoneAlreadyExist);
                        }

                        if !self.state.board.is_legal_move(pos.unwrap()) {
                            return Err(GameError::ForbiddenMove);
                        }

                        self.state.set_mut(pos.unwrap())
                    }
                }

                match self.state.board.find_winner() {
                    Some(winner) =>
                        message_sender.message(Message::Finished(message::GameResult::Win(winner))),
                    None => {
                        if self.state.board.stones == pos::U8_BOARD_SIZE {
                            message_sender.message(Message::Finished(message::GameResult::Full));
                        } else if self.config.draw_condition.is_some_and(|draw_in|
                            draw_in >= self.state.history.len()
                        ) {
                            message_sender.message(Message::Finished(message::GameResult::Draw));
                        }
                    }
                }
            },
            Command::Set { pos, color } => {
                if !self.state.board.is_pos_empty(pos) {
                    return Err(GameError::StoneAlreadyExist);
                }

                self.state.movegen_window.imprint_window_mut(pos);

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

                        if let Some(time_management) = &mut self.time_management
                            && let Some((index, time)) = time_management.time_history.iter()
                                .enumerate()
                                .find_map(|(index, (action, time))|
                                    (action == &pos.into()).then_some((index, *time))
                                )
                        {
                            time_management.time_history.remove(index);
                            time_management.time_manager.append_mut(time);
                        }

                    },
                    Some(_) => return Err(GameError::StoneColorMismatch),
                    None => return Err(GameError::StoneDoesNotExist),
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

                        if self.executed_moves.is_hot_idx(self.state.history.len()) {
                            self.executed_moves.unset_idx_mut(self.state.history.len());

                            if let Some(time_management) = &mut self.time_management
                                && let Some((pos, time)) = time_management.time_history.pop()
                                && pos == action
                            {
                                time_management.time_manager.append_mut(time);
                            }
                        }
                    }
                }
            },
            Command::Load(boxed) => {
                let (board, history) = *boxed;

                let movegen_window = (&board.hot_field).into();
                let move_scores = (&board.hot_field).into();

                self.state = GameState {
                    board,
                    history,
                    movegen_window,
                    move_scores,
                };

                self.tt.clear_mut(self.config.workers.into());
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
                    self.state.board.opponent_color()
                };

                self.state.board.batch_set_each_color_mut(
                    black_stones.into_boxed_slice(),
                    white_stones.into_boxed_slice(),
                    player
                );
            }
            Command::TurnTime(time) => {
                let Some(time_management) = &mut self.time_management else {
                    return Err(GameError::NoTimeManagement);
                };

                time_management.time_manager.turn = time;
            },
            Command::IncrementTime(time) => {
                let Some(time_management) = &mut self.time_management else {
                    return Err(GameError::NoTimeManagement);
                };

                time_management.time_manager.increment = time;
            },
            Command::TotalTime(time) => {
                let Some(time_management) = &mut self.time_management else {
                    return Err(GameError::NoTimeManagement);
                };

                time_management.time_manager.total_remaining = time;
            },
            Command::ConsumeTime(time) => {
                let Some(time_management) = &mut self.time_management else {
                    return Err(GameError::NoTimeManagement);
                };

                time_management.time_manager.consume_mut(time);
            }
            Command::MaxNodes { in_1k } => {
                self.config.max_nodes_in_1k = in_1k;
            },
            Command::Workers(workers) => {
                self.config.workers = workers;
            },
            Command::MaxMemory(size) => {
                const HEAP_MEMORY_MARGIN_IN_KIB: usize = 1024 * 10;

                self.tt.resize_mut(ByteSize::from_kib(size.kib() + HEAP_MEMORY_MARGIN_IN_KIB));
            },
            Command::Rule(kind) => {
                self.config.rule_kind = kind;
            },
        };

        Ok(())
    }

    pub fn commands(&mut self, message_sender: &MessageSender, commands: Vec<Command>) -> Result<(), GameError> {
        commands.into_iter()
            .try_for_each(|command| self.command(message_sender, command))
    }

    fn next_computing_resource(&self) -> ComputingResource {
        ComputingResource {
            workers: self.config.workers.get(),
            tt_size: self.config.tt_size,
            time: self.time_management.as_ref().map(|time_management|
                time_management.time_manager.next_running_time()
            ),
            nodes_in_1k: self.config.max_nodes_in_1k,
        }
    }

    pub fn launch(&mut self, response_sender: impl ResponseSender, aborted: Arc<AtomicBool>) -> BestMove {
        let computing_resource = self.next_computing_resource();

        if let Some(time_management) = &mut self.time_management {
            time_management.time_manager.append_mut(time_management.time_manager.increment);
        }

        let started_time = std::time::Instant::now();

        aborted.store(false, Ordering::Relaxed);
        let global_counter_in_1k = Arc::new(AtomicUsize::new(0));

        response_sender.response(Response::Begins(computing_resource));

        let tt_view = self.tt.view();

        let (main_td, score) = std::thread::scope(|s| {
            let state = self.state;

            let mut main_td = ThreadData::new(
                MainThread::new(
                    response_sender,
                    started_time,
                    computing_resource.time,
                ),
                0,
                self.config,
                tt_view,
                HistoryTable {},
                &aborted, &global_counter_in_1k,
            );

            let handle = s.spawn(move || {
                let score = search::iterative_deepening::<{ RuleKind::Renju }, MainThread<_>>(
                    &mut main_td, state
                );

                (main_td, score)
            });

            for tid in 1 ..self.config.workers.get() {
                let mut worker_td = ThreadData::new(
                    WorkerThread, tid,
                    self.config,
                    tt_view,
                    HistoryTable {},
                    &aborted, &global_counter_in_1k
                );

                s.spawn(move || {
                    search::iterative_deepening::<{ RuleKind::Renju }, WorkerThread>(
                        &mut worker_td, state
                    );
                });
            }

            handle.join().unwrap()
        });

        self.tt.increase_age();
        self.ht = *main_td.ht;

        self.executed_moves.set_idx_mut(self.state.history.len());

        let time_elapsed = started_time.elapsed();

        if let Some(time_management) = &mut self.time_management {
            time_management.time_manager.consume_mut(time_elapsed);
            time_management.time_history.push((main_td.best_move, time_elapsed));
        }

        BestMove {
            hash: self.state.board.hash_key,
            pos: main_td.best_move,
            score,
            total_nodes_in_1k: main_td.batch_counter.count_global_in_1k(),
            time_elapsed,
        }
    }

}
