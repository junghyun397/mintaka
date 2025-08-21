use clap::{Parser, ValueEnum};
use mintaka::config::Config;
use mintaka::game_state::GameState;
use mintaka::time_manager::TimeManager;
use rusty_renju::board::Board;
use rusty_renju::history::History;
use std::time::Duration;

#[derive(Copy, Clone, Eq, PartialEq, Debug, ValueEnum)]
pub enum Mode {
    SelfPlay,
    TextProtocol,
}

#[derive(Clone, Parser)]
#[clap(
    disable_help_flag = true,
)]
pub struct Preference {
    #[arg(short, long, value_enum, default_value_t = Mode::TextProtocol)]
    pub mode: Mode,
    #[arg(short, long)]
    pub board: Option<Board>,
    #[arg(short, long)]
    pub history: Option<History>,
    #[arg(short, long)]
    pub time_in_millis: Option<u64>,
    #[arg(short, long)]
    pub nodes_in_1k: Option<usize>,
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
        if self.mode == Mode::SelfPlay {
            self.game_state = Some(if let Some(history) = self.history {
                history.into()
            } else if let Some(board) = self.board {
                board.into()
            } else {
                GameState::default()
            });
        }

        if let Some(time_in_millis) = self.time_in_millis {
            self.default_config.initial_time_manager = Some(TimeManager {
                total_remaining: Duration::MAX,
                increment: Duration::ZERO,
                turn: Duration::from_millis(time_in_millis),
            });
        }

        self.default_config.max_nodes_in_1k = self.nodes_in_1k;

        self.default_config = self.default_config.validate().unwrap();
    }

}
