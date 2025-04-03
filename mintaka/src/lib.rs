#![allow(incomplete_features)]
#![feature(adt_const_params)]
#![feature(portable_simd)]
#![feature(avx512_target_feature)]
#![cfg_attr(target_arch = "aarch64", feature(stdarch_aarch64_prefetch))]

pub mod config;
pub mod memo;
pub mod eval;
pub mod endgame;
pub mod tablebase;
pub mod utils;
pub mod protocol;
pub mod search;
pub mod principal_variation;
pub mod nnue;
pub mod thread_data;
pub mod thread_type;
pub mod batch_counter;
pub mod search_frame;
pub mod game_state;
pub mod movegen;
pub mod game_agent;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
