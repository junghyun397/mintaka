use mintaka::config::Config;
use mintaka::endgame;
use mintaka::memo::history_table::HistoryTable;
use mintaka::memo::transposition_table::TranspositionTable;
use mintaka::search_limit::SearchLimit;
use mintaka::thread_data::ThreadData;
use rusty_renju::board::Board;
use rusty_renju::memo::abstract_transposition_table::AbstractTranspositionTable;
use rusty_renju::notation::pos;
use std::env;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::time::Instant;

fn main() -> Result<(), &'static str> {
    let mut board = env::args().collect::<Vec<String>>()
        .join(" ")
        .parse::<Board>()
        .map_err(|_| "invalid argument")
        ?;

    let config = Config::default();

    let tt = TranspositionTable::new_with_size(512);
    let ht = HistoryTable {};
    let global_counter_in_1k = AtomicUsize::new(0);
    let global_aborted = AtomicBool::new(false);
    let mut td = ThreadData::new(0, config, SearchLimit::Infinite, &tt, ht, &global_aborted, &global_counter_in_1k);

    let instant = Instant::now();
    let vcf_result = endgame::vcf::vcf_sequence(&mut td, &board, pos::U8_BOARD_SIZE)
        .ok_or("solution not exists")
        ?;
    let time = instant.elapsed();

    board.batch_set_mut(&vcf_result.clone().into_boxed_slice());

    println!("{}", board.to_string_with_move_marker(*vcf_result.last().unwrap()));
    println!("sequence: {:?}", vcf_result);
    println!("length: {}", vcf_result.len());
    println!("time: {:?}", time);
    println!("hash usage: {}", tt.hash_usage());
    println!("nodes: {}", td.batch_counter.count_local_total());

    Ok(())
}
