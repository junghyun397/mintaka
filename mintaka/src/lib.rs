#![feature(adt_const_params)]
#![feature(portable_simd)]
#![allow(incomplete_features)]

pub mod board;
pub mod board_io;
pub mod slice;

pub mod cache;
pub mod movegen;
pub mod notation;
pub mod game;
pub mod utils;
pub mod opening;
pub mod formation;
pub mod pattern;
pub mod board_iter;
