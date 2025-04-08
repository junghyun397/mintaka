use crate::config::Config;
use crate::game_state::GameState;
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TranspositionTable;
use crate::protocol::command::Command;
use crate::protocol::message::ResponseSender;
use crate::protocol::response::Response;
use crate::search;
use crate::search_limit::SearchLimit;
use crate::thread_data::ThreadData;
use crate::thread_type::{MainThread, ThreadType, WorkerThread};
use crate::utils::time_manager::TimeManager;
use rusty_renju::history::History;
use rusty_renju::memo::abstract_transposition_table::AbstractTranspositionTable;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::rule::RuleKind;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

pub struct GameAgent {
    pub config: Config,
    pub state: GameState,
    pub history: History,
    pub rule: RuleKind,
    time_manager: TimeManager,
    tt: TranspositionTable,
    ht: HistoryTable,
    aborted: Arc<AtomicBool>,
}

impl GameAgent {

    pub fn new(config: Config, aborted: Arc<AtomicBool>) -> Self {
        Self {
            config,
            state: GameState::default(),
            rule: RuleKind::Renju,
            history: History::default(),
            time_manager: TimeManager::default(),
            tt: TranspositionTable::new_with_size(1024 * 16),
            ht: HistoryTable {},
            aborted,
        }
    }

    pub fn command(&mut self, command: Command) -> Result<(), &'static str> {
        match command {
            Command::Play(action) => {
                match action {
                    MaybePos::NONE => {
                        self.state.board.pass_mut();
                        self.history.pass_mut();
                    },
                    pos => {
                        self.state.board.set_mut(pos.unwrap());
                        self.history.set_mut(pos.unwrap());
                    }
                }
            },
            Command::Set { pos, color } => {
                if !self.state.board.is_pos_empty(pos) {
                    return Err("stone already exists");
                }

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
            },
            Command::TurnTime(time) => {
                self.time_manager.turn = time;
            },
            Command::IncrementTime(time) => {
                self.time_manager.increment = time;
            },
            Command::MaxNodes { in_1k } => {

            },
            Command::Workers(workers) => {
                self.config.workers = workers;
            },
            Command::Rule(kind) => {
                self.rule = kind;
            },
            Command::MaxMemory { in_kib } => {
                const HEAP_MEMORY_MARGIN_IN_KIB: usize = 1024 * 10;

                self.tt.resize_mut(in_kib + HEAP_MEMORY_MARGIN_IN_KIB);
            },
        };

        Ok(())
    }

    pub fn launch(&mut self, response_sender: ResponseSender) {
        self.aborted.store(false, Ordering::Relaxed);
        let global_counter_in_1k = AtomicUsize::new(0);

        let running_time = self.time_manager.next_running_time();
        self.time_manager.consume_mut(running_time);

        std::thread::scope(|s| {
            let mut state = self.state;

            let mut main_td = ThreadData::new(
                MainThread::new(
                    response_sender,
                    std::time::Instant::now(),
                    SearchLimit::Time { finish_at: running_time }
                ),
                0,
                self.config,
                self.tt.view(),
                self.ht.clone(),
                &self.aborted, &global_counter_in_1k,
            );

            s.spawn(move || {
                let score = search::iterative_deepening::<{ RuleKind::Renju }, _>(&mut main_td, &mut state);

                main_td.thread_type.make_response(||
                    Response::BestMove(main_td.best_move.unwrap(), score as f32)
                );
            });

            for tid in 1 .. self.config.workers.get() {
                let mut worker_td = ThreadData::new(
                    WorkerThread, tid,
                    self.config,
                    self.tt.view(),
                    self.ht.clone(),
                    &self.aborted, &global_counter_in_1k
                );

                let mut state = self.state;

                s.spawn(move || {
                    search::iterative_deepening::<{ RuleKind::Renju }, _>(&mut worker_td, &mut state);
                });
            }
        });

        self.tt.increase_age();
    }

    pub fn abort(&self) {
        self.aborted.store(true, Ordering::Relaxed);
    }

}
