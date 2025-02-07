use mintaka::endgame;
use mintaka::memo::transposition_table::TranspositionTable;
use mintaka::thread_data::ThreadData;
use rusty_renju::board::Board;
use rusty_renju::memo::abstract_transposition_table::AbstractTranspositionTable;
use std::env;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::time::Instant;

fn main() -> Result<(), &'static str> {
    let mut board = env::args().collect::<Vec<String>>()
        .join(" ")
        .parse::<Board>()
        ?;

    let global_counter = AtomicUsize::new(0);
    let global_aborted = AtomicBool::new(false);
    let mut td = ThreadData::new(&global_aborted, &global_counter);
    let mut tt = TranspositionTable::new_with_size(1);

    let instant = Instant::now();
    let vcf_result = endgame::vcf::vcf_sequence(&tt, &mut td, &board, u8::MAX)
        .ok_or("solution not exists.")?;
    let time = instant.elapsed();

    board.batch_set_mut(&vcf_result.clone().into_boxed_slice());

    println!("{}", board.to_string_with_move_marker(*vcf_result.last().unwrap()));
    println!("sequence: {:?}", vcf_result);
    println!("length: {}", vcf_result.len());
    println!("time: {:?}", time);
    println!("hash usage: {}", tt.hash_usage());
    println!("nodes: {}", td.batch_counter.total_local_count());

    Ok(())
}
