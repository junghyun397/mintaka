use clap::Parser;
use mintaka::config::Config;
use mintaka::game_state::GameState;
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::utils::byte_size::ByteSize;
use std::time::Duration;

#[derive(Clone, Parser)]
#[clap(
    disable_help_flag = true,
)]
pub struct Preference {
    #[arg(short, long)]
    pub board: Option<Board>,
    #[arg(short, long)]
    pub history: Option<History>,
    #[arg(short, long, num_args = 3)]
    pub time: Option<Vec<u64>>,
    #[arg(long)]
    pub dynamic_time: bool,
    #[arg(short, long)]
    pub nodes_in_1k: Option<u64>,
    #[arg(short, long)]
    pub memory_in_mib: Option<u64>,
    #[arg(short, long)]
    pub workers: Option<u32>,
    #[arg(short, long)]
    pub pondering: bool,
    #[arg(short, long)]
    pub command_sequence: Option<String>,
    #[clap(skip)]
    pub game_state: Option<GameState>,
    #[clap(skip)]
    pub default_config: Config,
}

impl Preference {

    pub fn parse() -> Self {
        let mut pref = Self::parse_from(std::env::args());

        pref.init();

        pref
    }

    fn init(&mut self) {
        if let Some(history) = self.history {
            self.game_state = Some(history.into());
        } else if let Some(board) = self.board {
            let history = (&board).try_into().unwrap();
            self.game_state = Some(GameState::from_board_and_history(board, history));
        }

        if let Some(&[total_time_in_ms, increment_time_in_ms, turn_time_in_ms]) = self.time.as_deref() {
            self.default_config.initial_timer.total_remaining = Duration::from_millis(total_time_in_ms);
            self.default_config.initial_timer.increment = Duration::from_millis(increment_time_in_ms);
            self.default_config.initial_timer.turn = Duration::from_millis(turn_time_in_ms);
        }

        self.default_config.dynamic_time = self.dynamic_time;

        self.default_config.pondering = self.pondering;

        if let Some(memory_in_mib) = self.memory_in_mib {
            self.default_config.tt_size = ByteSize::from_mib(memory_in_mib as usize);
        }

        match self.workers {
            Some(workers) => self.default_config.workers = workers,
            None => self.default_config.workers = num_cpus::get() as u32,
        }

        self.default_config.max_nodes_in_1k = self.nodes_in_1k;

        self.default_config = self.default_config.validate().unwrap();
    }

}
