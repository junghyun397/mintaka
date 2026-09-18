use mintaka::config::Config;
use mintaka::game_state::{GameState, GameStateData};
use mintaka::protocol::time::{TimeUnit, TimeValue};
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::utils::byte_size::ByteSize;

pub struct Params<const R: RuleKind> {
    pub command_sequence: Option<String>,
    pub game_state: Option<GameState<R>>,
    pub config: Config,
}

impl<const R: RuleKind> Params<R> {
    pub fn parse() -> Self {
        Self::parse_args().unwrap_or_else(|error| {
            eprintln!("{error}");
            std::process::exit(1);
        })
    }

    fn parse_args() -> Result<Self, String> {
        let mut args = std::env::args().skip(1);
        let mut board = None;
        let mut history = None;
        let mut time_total = None;
        let mut time_increment = None;
        let mut time_turn = None;
        let mut command_sequence = None;
        let mut config = Config::default();

        config.workers = std::thread::available_parallelism()
            .map_or_else(|_| 1, |n| n.get()) as u32;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--board" | "-b" => {
                    board = Some(args.next().ok_or("board not provided.")?
                        .parse::<Board<R>>()?);
                }
                "--history" | "-h" => {
                    history = Some(args.next().ok_or("history not provided.")?
                        .parse::<History>()?);
                }
                "--time-unit" | "-u" => {
                    config.initial_timer.time_unit = args.next().ok_or("time unit not provided.")?
                        .parse::<TimeUnit>()?;
                }
                "--time-total" => {
                    time_total = Some(args.next().ok_or("total time not provided.")?
                        .parse::<u32>()
                        .map_err(|_| "invalid total time.")?);
                }
                "--time-increment" => {
                    time_increment = Some(args.next().ok_or("increment time not provided.")?
                        .parse::<u32>()
                        .map_err(|_| "invalid increment time.")?);
                }
                "--time-turn" => {
                    time_turn = Some(args.next().ok_or("turn time not provided.")?
                        .parse::<u32>()
                        .map_err(|_| "invalid turn time.")?);
                }
                "--memory-in-mib" | "-m" => {
                    let memory_in_mib = args.next().ok_or("memory not provided.")?
                        .parse::<u32>()
                        .map_err(|_| "invalid memory size.")?;

                    config.tt_size = ByteSize::from_mib(memory_in_mib as u64);
                }
                "--workers" | "-w" => {
                    config.workers = args.next().ok_or("workers not provided.")?
                        .parse::<u32>()
                        .map_err(|_| "invalid workers number.")?;
                }
                "--pondering" | "-p" => {
                    config.pondering = true;
                }
                "--command-sequence" => {
                    command_sequence = Some(args.next().ok_or("command sequence not provided.")?);
                }
                _ => return Err(format!("unknown option: {arg}")),
            }
        }

        let game_state = if let Some(history) = history {
            Some(history.into())
        } else if let Some(board) = board {
            Some(GameStateData { board_data: (&board).into(), history: board.build_history() }.into())
        } else {
            None
        };

        let unit = config.initial_timer.time_unit;

        if let Some(total) = time_total {
            config.initial_timer.total_remaining = (total != 0).then_some(TimeValue::from_value(total as u64, unit));
        }
        if let Some(increment) = time_increment {
            config.initial_timer.increment = TimeValue::from_value(increment as u64, unit);
        }
        if let Some(turn) = time_turn {
            config.initial_timer.turn = (turn != 0).then_some(TimeValue::from_value(turn as u64, unit));
        }

        Ok(Self {
            command_sequence,
            game_state,
            config: config.validate().map_err(|error| error.to_string())?,
        })
    }
}
