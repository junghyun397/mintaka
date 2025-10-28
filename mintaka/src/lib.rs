#![allow(incomplete_features)]
#![feature(adt_const_params)]
#![feature(portable_simd)]
#![cfg_attr(target_arch = "aarch64", feature(stdarch_aarch64_prefetch))]
extern crate core;

pub mod batch_counter;
pub mod config;
pub mod eval;
pub mod game_agent;
pub mod game_state;
pub mod memo;
pub mod movegen;
pub mod principal_variation;
pub mod protocol;
pub mod search;
pub mod search_frame;
pub mod tablebase;
pub mod thread_data;
pub mod thread_type;
pub mod time_manager;
pub mod value;
pub mod utils;
pub mod search_endgame;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
