use crate::config::Config;
use crate::game_state::GameState;
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TranspositionTable;
use crate::protocol::command::Command;
use crate::protocol::response::Response;
use crate::search;
use crate::thread_data::ThreadData;
use crate::thread_type::{MainThread, WorkerThread};
use crate::utils::time_manager::TimeManager;
use rusty_renju::history::History;
use rusty_renju::memo::abstract_transposition_table::AbstractTranspositionTable;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::Pos;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

pub struct GameAgent {
    pub config: Config,
    pub own_color: Color,
    pub state: GameState,
    history: History,
    time_manager: TimeManager,
    tt: TranspositionTable,
    ht: HistoryTable,
    global_aborted: AtomicBool,
}

impl GameAgent {

    pub fn new(config: Config) -> Self {
        Self {
            own_color: Color::Black,
            config,
            state: GameState::default(),
            history: History::default(),
            time_manager: TimeManager::default(),
            tt: TranspositionTable::new_with_size(1024 * 16),
            ht: HistoryTable {},
            global_aborted: AtomicBool::new(false),
        }
    }

    pub fn command(&mut self, command: Command) {
        match command {
            Command::Status => {
                todo!()
            }
            Command::Abort => {
                self.global_aborted.store(true, Ordering::Relaxed);
            }
        }
    }

    pub fn launch(&mut self) -> std::sync::mpsc::Receiver<Response> {
        self.tt.increase_age();

        self.global_aborted.store(true, Ordering::Relaxed);
        let global_counter_in_1k = AtomicUsize::new(0);

        let (response_sender, response_receiver) = std::sync::mpsc::channel();

        let running_time = self.time_manager.next_running_time();
        self.time_manager.consume_mut(running_time);

        let mut main_td = ThreadData::new(
            MainThread {
                response_channel: response_sender.clone(),
                start_time: std::time::Instant::now(),
                time_limit: running_time,
            },
            0,
            self.config,
            self.tt.view(),
            self.ht.clone(),
            &self.global_aborted, &global_counter_in_1k,
        );

        std::thread::scope(|s| {

            s.spawn(|| {
                let (best_move, score) = search::iterative_deepening(&mut main_td, &mut self.state.clone());

                response_sender.send(Response::BestMove(best_move, score)).expect("sender channel closed.");
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

        response_receiver
    }

    pub fn set(&mut self, pos: Pos, color: Color) {
        self.state.board.player_color = color;
        self.state.board.set_mut(pos);
    }

    pub fn unset(&mut self, pos: Pos, color: Color) {
        self.state.board.player_color = !color;
        self.state.board.unset_mut(pos);
    }

    pub fn set_remaining_time(&mut self, remaining_time: Duration) {
        self.time_manager.total_remaining = remaining_time;
    }

    pub fn set_overhead_time(&mut self, overhead_time: Duration) {
        self.time_manager.overhead = overhead_time;
    }

    pub fn resize_tt(&mut self, size_in_kib: usize) {
        const MEMORY_MARGIN_IN_KIB: usize = 1024 * 10;

        self.tt.resize_mut(size_in_kib - MEMORY_MARGIN_IN_KIB);
    }

}
