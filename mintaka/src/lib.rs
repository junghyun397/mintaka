#![allow(incomplete_features)]
#![feature(portable_simd)]
#![feature(adt_const_params)]

extern crate core;

pub mod board;
pub mod board_io;
pub mod slice;
pub mod memo;
pub mod movegen;
pub mod notation;
pub mod game;
pub mod utils;
pub mod opening;
pub mod pattern;
pub mod slice_pattern;
pub mod board_iter;
pub mod bitfield;
