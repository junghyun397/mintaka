use crate::config::Config;
use crate::game_state::GameState;
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TranspositionTable;
use crate::protocol::command::Command;
use crate::protocol::message::ResponseSender;
use crate::protocol::response::Response;
use crate::search;
use crate::thread_data::ThreadData;
use crate::thread_type::{MainThread, ThreadType, WorkerThread};
use crate::utils::time_manager::TimeManager;
use rusty_renju::history::{Action, History};
use rusty_renju::memo::abstract_transposition_table::AbstractTranspositionTable;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::rule::RuleKind;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};

pub struct GameAgent {
    pub config: Config,
    pub own_color: Color,
    pub state: GameState,
    pub rule: RuleKind,
    history: History,
    time_manager: TimeManager,
    tt: TranspositionTable,
    ht: HistoryTable,
    global_aborted: AtomicBool,
    best_move: AtomicU8,
}

pub enum GameCommand {
    Command(Command),
    Launch,
    Abort,
    Quite,
}

impl GameAgent {

    pub fn new(config: Config) -> Self {
        Self {
            own_color: Color::Black,
            config,
            state: GameState::default(),
            rule: RuleKind::Renju,
            history: History::default(),
            time_manager: TimeManager::default(),
            tt: TranspositionTable::new_with_size(1024 * 16),
            ht: HistoryTable {},
            global_aborted: AtomicBool::new(false),
            best_move: AtomicU8::new(Pos::INVALID.idx()),
        }
    }

    pub fn command(&mut self, command: Command) {
        match command {
            Command::Play(action) => {
                match action {
                    Action::Move(pos) => {
                        self.state.board.set_mut(pos);
                        self.history.play_mut(pos);
                    }
                    Action::Pass => {
                        self.state.board.pass_mut();
                        self.history.pass_mut();
                    }
                }
            },
            Command::Set { pos, color } => {
                self.state.board.set_mut(pos);
                self.state.board.switch_player_mut();
            },
            Command::Unset { pos, color } => {
                self.state.board.unset_mut(pos);
                self.state.board.switch_player_mut();
            },
            Command::Undo => {
                self.state.board.unset_mut(todo!());
            },
            Command::Switch => {
                self.own_color = !self.own_color;
                self.state.board.switch_player_mut();
            }
            Command::BatchSet { black_stones, white_stones, player_color } => {
                self.state.board.batch_set_each_color_mut(black_stones, white_stones, player_color);
            },
            Command::TotalTime(time) => {
                self.time_manager.total_remaining = time;
            },
            Command::TurnTime(time) => {
                self.time_manager.turn = time;
            },
            Command::IncrementTime(time) => {
                self.time_manager.increment = time;
            },
            Command::Rule(kind) => {
                self.rule = kind;
            },
            Command::MaxMemory { in_kib } => {
                const HEAP_MEMORY_MARGIN_IN_KIB: usize = 1024 * 10;

                self.tt.resize_mut(in_kib + HEAP_MEMORY_MARGIN_IN_KIB);
            },
        }
    }

    pub fn launch(&mut self, response_sender: ResponseSender) {
        self.global_aborted.store(true, Ordering::Relaxed);
        let global_counter_in_1k = AtomicUsize::new(0);

        let running_time = self.time_manager.next_running_time();
        self.time_manager.consume_mut(running_time);

        let mut main_td = ThreadData::new(
            MainThread::new(
                response_sender,
                std::time::Instant::now(),
                running_time
            ),
            0,
            self.config,
            self.tt.view(),
            self.ht.clone(),
            &self.global_aborted, &global_counter_in_1k,
        );

        std::thread::scope(|s| {
            let mut state = self.state.clone();
            let mut atomic_best_move = &self.best_move;

            s.spawn(move || {
                let (best_move, score) = search::iterative_deepening(&mut main_td, &mut state);

                atomic_best_move.store(0, Ordering::Relaxed);

                main_td.thread_type.make_response(||
                    Response::BestMove(best_move, score)
                );
            });

            for tid in 1 .. self.config.workers.get() {
                let mut worker_td = ThreadData::new(
                    WorkerThread, tid,
                    self.config,
                    self.tt.view(),
                    self.ht.clone(),
                    &self.global_aborted, &global_counter_in_1k
                );

                let mut state = self.state.clone();

                s.spawn(move || {
                    search::iterative_deepening(&mut worker_td, &mut state);
                });
            }
        });

        self.tt.increase_age();
    }

    pub fn abort(&self) {
        self.global_aborted.store(true, Ordering::Relaxed);
    }

}
