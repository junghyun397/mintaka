use crate::config::Config;
use crate::game_state::GameState;
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TranspositionTable;
use crate::protocol::command::Command;
use crate::protocol::response::Response;
use crate::protocol::runtime_command::RuntimeCommand;
use crate::search;
use crate::thread_data::ThreadData;
use crate::thread_type::{MainThread, WorkerThread};
use crate::utils::time_manager::TimeManager;
use rusty_renju::history::{Action, History};
use rusty_renju::memo::abstract_transposition_table::AbstractTranspositionTable;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::rule::RuleKind;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::Duration;

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
            }
            Command::Set { pos, color } => {
                todo!()
            }
            Command::Unset { pos, color } => {
                todo!()
            }
            Command::Undo => {
                todo!()
            }
            Command::TotalTime(time) => {
                self.time_manager.total_remaining = time;
            }
            Command::TurnTime(time) => {
                self.time_manager.turn = time;
            }
            Command::Rule(kind) => {
                self.rule = kind;
            }
        }
    }

    pub fn launch(&mut self, runtime_commander: mpsc::Receiver<RuntimeCommand>) -> mpsc::Receiver<Response> {
        self.tt.increase_age();

        self.global_aborted.store(true, Ordering::Relaxed);
        let global_counter_in_1k = AtomicUsize::new(0);

        let (response_sender, response_receiver) = mpsc::channel();

        let running_time = self.time_manager.next_running_time();
        self.time_manager.consume_mut(running_time);

        let mut main_td = ThreadData::new(
            MainThread::new(
                runtime_commander,
                response_sender.clone(),
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

    pub fn recent_pos(&self) -> Option<Pos> {
        self.history.0.last().and_then(|action| {
            if let Action::Move(pos) = action {
                Some(*pos)
            } else {
                None
            }
        })
    }

    pub fn play(&mut self, pos: Pos) {
        self.state.board.set_mut(pos);
    }

    pub fn undo(&mut self) {
        if let Some(action) = self.history.0.pop() {
            match action {
                Action::Move(pos) => {
                    self.state.board.unset_mut(pos);
                },
                Action::Pass => {
                    self.state.board.switch_player_mut();
                }
            }
        }
    }

    pub fn pass(&mut self) {
        self.state.board.pass_mut();
        self.history.pass_mut();
    }

    pub fn switch_color(&mut self) {
        self.own_color = !self.own_color;
    }

    pub fn batch_set(&mut self, black_stones: Box<[Pos]>, white_stones: Box<[Pos]>, player_color: Color) {
        self.state.board.batch_set_each_color_mut(black_stones, white_stones, player_color)
    }

    pub fn set_remaining_time(&mut self, remaining_time: Duration) {
        self.time_manager.total_remaining = remaining_time;
    }

    pub fn set_turn_time(&mut self, turn_time: Duration) {
        self.time_manager.turn = turn_time;
    }

    pub fn resize_tt(&mut self, size_in_kib: usize) {
        const MEMORY_MARGIN_IN_KIB: usize = 1024 * 10;

        self.tt.resize_mut(size_in_kib - MEMORY_MARGIN_IN_KIB);
    }

}

pub struct RuntimeCommander<'a> {
    global_aborted: &'a AtomicBool,
}

impl RuntimeCommander<'_> {

    pub fn command(&self, command: RuntimeCommand) {
        match command {
            RuntimeCommand::Status => {
                todo!()
            }
            RuntimeCommand::Abort => {
                self.global_aborted.store(true, Ordering::Relaxed);
            }
        }
    }

}
