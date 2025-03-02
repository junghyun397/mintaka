#![feature(test)]

extern crate test;

use indoc::indoc;
use mintaka::config::Config;
use mintaka::endgame::vcf;
use mintaka::memo::history_table::HistoryTable;
use mintaka::memo::transposition_table::TranspositionTable;
use mintaka::thread_data::ThreadData;
use mintaka::thread_type::ThreadType;
use rusty_renju::board::Board;
use std::sync::atomic::{AtomicBool, AtomicUsize};

fn setup_test(sample_board: &str) -> (Board, ThreadData<'static>) {
    let board = sample_board.parse::<Board>().unwrap();

    let config = Config::default();

    let tt = Box::leak(Box::new(TranspositionTable::new_with_size(512)));
    let ht = HistoryTable {};

    let global_counter_in_1k = Box::leak(Box::new(AtomicUsize::new(0)));
    let global_aborted = Box::leak(Box::new(AtomicBool::new(false)));

    let td = ThreadData::new(ThreadType::Main, 0, config, tt, ht, global_aborted, global_counter_in_1k);

    (board, td)
}
#[bench]
fn trap_vcf(b: &mut criterion::Bencher) {
    let (board, td) = setup_test(indoc! {"\
    "});

    b.iter(|| {

    })
}

#[bench]
fn deep_vcf(b: &mut criterion::Bencher) {
    let (board, td) = setup_test(indoc! {"\
           A B C D E F G H I J K L M N O
        15 O . . . X . . . . . . . X . X 15
        14 X . . . . O . . . O . . O . X 14
        13 . . . . . . . O . . . . . O . 13
        12 O . . . . . . . . . . X . . X 12
        11 X . . . . . . . . . . . O . . 11
        10 O . O . . . . . . . . . . . . 10
         9 O O X O . . . . X . . . O . . 9
         8 O . O O . . . X . O . . . . . 8
         7 . X . . . . . . . O . . X . . 7
         6 . . . . . . . . O . . . . . X 6
         5 X . . . . . . . . . . . X . X 5
         4 . . . . . . . . . . . . . X O 4
         3 X . . . . . . . . . . . . X . 3
         2 . . . . . . . X . . . . . . O 2
         1 X O O O . X . . X . X . . . . 1
           A B C D E F G H I J K L M N O
    "});

    b.iter(|| {
        let mut td = td.clone();
        vcf::vcf_search(&mut td, &board, u8::MAX);
    })
}
