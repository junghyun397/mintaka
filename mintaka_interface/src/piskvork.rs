use crate::piskvork_game_manager::PiskvorkGameManager;
use mintaka::config::Config;
use mintaka::memo::history_table::HistoryTable;
use mintaka::memo::transposition_table::TranspositionTable;
use mintaka::utils::time_manager::TimeManager;
use rusty_renju::board::Board;
use rusty_renju::memo::abstract_transposition_table::AbstractTranspositionTable;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::rule::RuleKind;
use std::time::Duration;

mod piskvork_game_manager;

enum PiskvorkResponse {
    Info(&'static str),
    Error(&'static str),
    Pos(Pos),
    Ok,
    None
}

fn main() -> Result<(), &'static str> {
    let manager = PiskvorkGameManager {};

    let mut config = Config::default();
    let mut time_manager = TimeManager::default();

    let mut transposition_table = TranspositionTable::new_with_size(1024 * 16);
    let mut history_table = HistoryTable {};

    let mut own_color = Color::Black;
    let mut board = Board::default();

    loop {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).map_err(|_| "failed to stdio")?;
        buf.make_ascii_uppercase();
        let args = buf.trim().split(' ').collect::<Vec<&str>>();

        if args.len() == 0 {
            continue;
        }

        let command = args[0];
        let parameters = &args[1..];

        // https://plastovicka.github.io/protocl2en.htm
        let piskvork_response = match command {
            "START" => {
                let size: usize = parameters[0].parse().map_err(|_| "size parsing failed.")?;
                if size == pos::U_BOARD_WIDTH {
                    PiskvorkResponse::Ok
                } else {
                    PiskvorkResponse::Error("unsupported size or other error")
                }
            },
            "RECTSTART" => {
                PiskvorkResponse::Error("rectangular board is not supported or other error")
            }
            "BEGIN" => {
                // launch
                PiskvorkResponse::Pos(Pos::from_str_unchecked("h8"))
            },
            "TURN" => {
                let pos = Pos::from_cartesian(
                    parameters[0].parse::<u8>().map_err(|_| "row parsing failed.")? - 1,
                    parameters[1].parse::<u8>().map_err(|_| "column parsing failed.")? - 1
                );

                // launch
                PiskvorkResponse::Pos(Pos::from_str_unchecked("h8"))
            },
            "TAKEBACK" => {
                let pos = Pos::from_cartesian(
                    parameters[0].parse::<u8>().map_err(|_| "row parsing failed.")? - 1,
                    parameters[1].parse::<u8>().map_err(|_| "column parsing failed.")? - 1
                );

                // undo
                PiskvorkResponse::Ok
            },
            "BOARD" => {
                const DONE_TOKEN: &str = "DONE";

                loop {
                    let mut buf = String::new();
                    std::io::stdin().read_line(&mut buf).map_err(|_| "failed to stdio")?;

                    if buf.trim() == DONE_TOKEN {
                        break;
                    }

                    let [row, col, color]: [&str; 3] = buf.trim()
                        .split(',')
                        .collect::<Vec<&str>>()
                        .try_into()
                        .map_err(|_| "token parsing failed.")
                        ?;

                    let pos = Pos::from_cartesian(
                        row.parse::<u8>().map_err(|e| "row parsing failed.")? - 1,
                        col.parse::<u8>().map_err(|e| "column parsing failed.")? - 1,
                    );

                    let color = match color {
                        "1" => own_color,
                        "2" => !own_color,
                        "3" => own_color,
                        &_ => {
                            return Err("unknown color token.");
                        }
                    };

                    board.set_mut(pos);
                }

                // launch
                PiskvorkResponse::Pos(Pos::from_str_unchecked("h8"))
            },
            "END" => {
                PiskvorkResponse::None
            },
            "INFO" => {
                fn parse_time(parameters: &[&str]) -> Result<Duration, &'static str> {
                    parameters.get(0)
                        .ok_or("missing info value.")
                        .and_then(|token| token
                            .parse::<u64>()
                            .map_err(|_| "time parsing failed.")
                        )
                        .map(Duration::from_millis)
                }

                match *args.get(0).ok_or("missing info key.")? {
                    "timeout_match" | "time_left" => {
                        time_manager.total_remaining = parse_time(parameters)?;

                        PiskvorkResponse::Ok
                    },
                    "timeout_turn" => {
                        time_manager.overhead = parse_time(parameters)?;

                        PiskvorkResponse::Ok
                    },
                    "max_memory" => {
                        const MEMORY_MARGIN_IN_KIB: usize = 1024 * 50;

                        let max_memory_in_bytes: usize =
                            parameters.get(0).expect("missing info value.").parse().unwrap();

                        transposition_table.resize_mut(max_memory_in_bytes / 1024 - MEMORY_MARGIN_IN_KIB);

                        PiskvorkResponse::Ok
                    },
                    "game_type" => {
                        match parameters.get(1)
                            .expect("missing info value.")
                            .chars().next().unwrap()
                        {
                            '0' ..= '3' => PiskvorkResponse::Ok,
                            _ => PiskvorkResponse::Error("unknown game type."),
                        }
                    },
                    "rule" => {
                        match parameters.get(1)
                            .expect("missing info value.")
                            .parse::<usize>().unwrap()
                            .count_ones()
                        {
                            1 => {
                                config.rule_kind = RuleKind::FiveInARow;
                                PiskvorkResponse::Ok
                            },
                            4 => {
                                config.rule_kind = RuleKind::Renju;
                                PiskvorkResponse::Ok
                            }
                            _ => PiskvorkResponse::Error("unsupported rule."),
                        }
                    },
                    "evaluate" => {
                        PiskvorkResponse::Ok
                    },
                    "folder" => {
                        PiskvorkResponse::Ok
                    },
                    &_ => {
                        PiskvorkResponse::Error("unsupported info key.")
                    }
                }
            },
            "YXHASHCLEAR" => {
                PiskvorkResponse::None
            },
            "YXSHOWFORBID" => {
                PiskvorkResponse::None
            },
            "YXSHOWINFO" => {
                PiskvorkResponse::None
            },
            "ABOUT" =>
                PiskvorkResponse::Info(
                    "name=\"mintaka\",\
                    author=\"JeongHyeon Choi\",\
                    version=\"0.9.0 pre-alpha\",\
                    country=\"KOR\""
                ),
            &_ => PiskvorkResponse::Error("unknown command.")
        };

        match piskvork_response {
            PiskvorkResponse::Info(message) => {
                println!("INFO {}", message);
            },
            PiskvorkResponse::Error(message) => {
                println!("ERROR {}", message);
            },
            PiskvorkResponse::Ok => {
                println!("OK");
            },
            PiskvorkResponse::Pos(pos) => {
                println!("{} {}", pos.row() + 1, pos.col() + 1);
            },
            PiskvorkResponse::None => {},
        };
    }
}
