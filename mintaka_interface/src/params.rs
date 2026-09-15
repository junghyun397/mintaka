use argh::FromArgs;
use mintaka::config::Config;
use mintaka::game_state::{GameState, GameStateData};
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::utils::byte_size::ByteSize;
use mintaka::protocol::time::{TimeUnit, TimeValue};
use rusty_renju::notation::rule::RuleKind;

#[derive(Clone)]
pub struct Params<const R: RuleKind> {
    pub command_sequence: Option<String>,
    pub game_state: Option<GameState<R>>,
    pub config: Config,
}

#[derive(FromArgs)]
#[argh(description = "mintaka text protocol", help_triggers("--help"))]
struct Args<const R: RuleKind> {
    #[argh(option, short = 'b', description = "initial board")]
    board: Option<Board<R>>,
    #[argh(option, short = 'h', description = "initial move history")]
    history: Option<History>,
    #[argh(option, short = 'u', description = "time unit: Clock (milliseconds) or Nodes (kilo)")]
    unit: Option<TimeUnit>,
    #[argh(option, description = "total budget; 0 disables the limit")]
    time_total: Option<u32>,
    #[argh(option, description = "increment")]
    time_increment: Option<u32>,
    #[argh(option, description = "per-turn budget; 0 disables the limit")]
    time_turn: Option<u32>,
    #[argh(option, short = 'm', description = "transposition table size in MiB")]
    memory_in_mib: Option<u32>,
    #[argh(option, short = 'w', description = "number of worker threads")]
    workers: Option<u32>,
    #[argh(switch, short = 'p', description = "enable pondering")]
    pondering: bool,
    #[argh(option, description = "newline-separated commands to execute on startup")]
    command_sequence: Option<String>,
}

impl<const R: RuleKind> Params<R> {
    pub fn parse() -> Self {
        argh::from_env::<Args<R>>().into()
    }
}

impl<const R: RuleKind> From<Args<R>> for Params<R> {
    fn from(args: Args<R>) -> Self {
        let game_state = if let Some(history) = args.history {
            Some(history.into())
        } else if let Some(board) = args.board {
            let history = (&board).try_into().unwrap();
            Some(GameStateData { board_data: (&board).into(), history }.into())
        } else {
            None
        };

        let mut config = Config::default();

        if let Some(unit) = args.unit {
            config.initial_timer.time_unit = unit;
        }

        let unit = config.initial_timer.time_unit;

        if let Some(total) = args.time_total {
            config.initial_timer.total_remaining = (total != 0).then_some(TimeValue::from_value(total as u64, unit));
        }
        if let Some(increment) = args.time_increment {
            config.initial_timer.increment = TimeValue::from_value(increment as u64, unit);
        }
        if let Some(turn) = args.time_turn {
            config.initial_timer.turn = (turn != 0).then_some(TimeValue::from_value(turn as u64, unit));
        }

        config.pondering = args.pondering;

        if let Some(memory_in_mib) = args.memory_in_mib {
            config.tt_size = ByteSize::from_mib(memory_in_mib as u64);
        }

        config.workers = args.workers.unwrap_or_else(|| std::thread::available_parallelism()
            .map_or_else(|_| 1, |n| n.get()) as u32);

        Self {
            command_sequence: args.command_sequence,
            game_state,
            config: config.validate().unwrap(),
        }
    }
}
