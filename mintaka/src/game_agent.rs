use crate::config::{Config, SearchLimit};
use crate::game_state::GameState;
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TranspositionTable;
use crate::protocol::command::Command;
use crate::protocol::message::ResponseSender;
use crate::protocol::response;
use crate::protocol::response::Response;
use crate::search;
use crate::thread_data::ThreadData;
use crate::thread_type::{MainThread, ThreadType, WorkerThread};
use crate::utils::time_manager::TimeManager;
use rusty_renju::history::History;
use rusty_renju::memo::abstract_transposition_table::AbstractTranspositionTable;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::utils::byte_size::ByteSize;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

pub struct GameAgent {
    pub state: GameState,
    pub history: History,
    config: Config,
    time_manager: TimeManager,
    search_limit: SearchLimit,
    tt: TranspositionTable,
    ht: HistoryTable,
}

impl GameAgent {

    pub fn new(config: Config) -> Self {
        Self {
            state: GameState::default(),
            history: History::default(),
            config,
            time_manager: TimeManager::default(),
            search_limit: SearchLimit::Infinite,
            tt: TranspositionTable::new_with_size(ByteSize::from_mib(16)),
            ht: HistoryTable {},
        }
    }

    pub fn command(&mut self, response_sender: &ResponseSender, command: Command) -> Result<(), &'static str> {
        match command {
            Command::Play(action) => {
                match action {
                    MaybePos::NONE => self.state.pass_mut(),
                    pos => {
                        if !self.state.board.is_pos_empty(pos.unwrap()) {
                            return Err("stone already exists");
                        }

                        if !self.state.board.is_legal_move(pos.unwrap()) {
                            return Err("forbidden move");
                        }

                        self.state.set_mut(pos.unwrap())
                    }
                }

                match self.state.board.find_winner() {
                    Some(winner) =>
                        response_sender.response(Response::Finished(response::GameResult::Win(winner))),
                    None => {
                        if self.state.board.stones == pos::U8_BOARD_SIZE {
                            response_sender.response(Response::Finished(response::GameResult::Full));
                        } else if self.config.draw_condition.is_some_and(|draw_in|
                            draw_in >= self.state.history.len()
                        ) {
                            response_sender.response(Response::Finished(response::GameResult::Draw));
                        }
                    }
                }
            },
            Command::Set { pos, color } => {
                if !self.state.board.is_pos_empty(pos) {
                    return Err("stone already exists");
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
                    },
                    Some(_) => return Err("stone color mismatch"),
                    None => return Err("stone does not exist"),
                }
            },
            Command::Undo => {
                match self.history.pop_mut() {
                    None => return Err("no history to undo"),
                    Some(action) => match action {
                        MaybePos::NONE => {
                            self.state.board.switch_player_mut();
                        },
                        pos => {
                            self.state.board.unset_mut(pos.unwrap());
                        }
                    }
                }
            },
            Command::Load(boxed) => {
                let (board, history) = *boxed;

                let movegen_window = (&board.hot_field).into();
                let move_scores = board.hot_field.into();

                self.state = GameState {
                    board,
                    history,
                    movegen_window,
                    move_scores,
                };

                self.history = history;

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
            Command::TotalTime(time) => {
                self.time_manager.total_remaining = time;
                self.search_limit = SearchLimit::Time { turn_time: time };
            },
            Command::TurnTime(time) => {
                self.time_manager.turn = time;
                self.search_limit = SearchLimit::Time { turn_time: time };
            },
            Command::IncrementTime(time) => {
                self.time_manager.increment = time;
                self.search_limit = SearchLimit::Time { turn_time: time };
            },
            Command::MaxNodes { in_1k } => {
                self.search_limit = SearchLimit::Nodes { in_1k }
            },
            Command::Workers(workers) => {
                self.config.workers = workers;
            },
            Command::Rule(kind) => {
                self.config.rule_kind = kind;
            },
            Command::MaxMemory(size) => {
                const HEAP_MEMORY_MARGIN_IN_KIB: usize = 1024 * 10;

                self.tt.resize_mut(ByteSize::from_kib(size.kib() + HEAP_MEMORY_MARGIN_IN_KIB));
            },
        };

        Ok(())
    }

    pub fn commands(&mut self, response_sender: &ResponseSender, commands: Vec<Command>) -> Result<(), &'static str> {
        commands.into_iter()
            .try_for_each(|command| self.command(response_sender, command))
    }

    pub fn launch(mut self, response_sender: ResponseSender, aborted: Arc<AtomicBool>) -> Self {
        aborted.store(false, Ordering::Relaxed);
        let global_counter_in_1k = AtomicUsize::new(0);

        self.time_manager.append_mut(self.time_manager.increment);
        let running_time = self.time_manager.next_running_time();
        let started_time = std::time::Instant::now();

        response_sender.response(Response::Begins {
            workers: self.config.workers.get(),
            running_time,
            tt_size: self.tt.size(),
        });

        std::thread::scope(|s| {
            let mut state = self.state;

            let mut main_td = ThreadData::new(
                MainThread::new(
                    response_sender.clone(),
                    started_time,
                    SearchLimit::Time { turn_time: running_time }
                ),
                0,
                self.config,
                self.tt.view(),
                self.ht.clone(),
                &aborted, &global_counter_in_1k,
            );

            s.spawn(move || {
                let score = search::iterative_deepening::<{ RuleKind::Renju }, MainThread>(
                    &mut main_td, &mut state
                );

                main_td.thread_type.make_response(||
                    Response::BestMove {
                        best_move: main_td.best_move.unwrap(),
                        score,
                        total_nodes_in_1k: main_td.batch_counter.count_global_in_1k(),
                        time_elapsed: started_time.elapsed(),
                    }
                );
            });

            for tid in 1 .. self.config.workers.get() {
                let mut worker_td = ThreadData::new(
                    WorkerThread, tid,
                    self.config,
                    self.tt.view(),
                    self.ht.clone(),
                    &aborted, &global_counter_in_1k
                );

                let mut state = self.state;

                s.spawn(move || {
                    search::iterative_deepening::<{ RuleKind::Renju }, WorkerThread>(
                        &mut worker_td, &mut state
                    );
                });
            }
        });

        self.tt.increase_age();
        self.time_manager.consume_mut(started_time.elapsed());

        self
    }

}
