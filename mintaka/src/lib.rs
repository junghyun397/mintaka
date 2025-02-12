#![allow(incomplete_features)]
#![feature(portable_simd)]
#![feature(adt_const_params)]

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
pub mod launch;
pub mod thread_type;
pub mod batch_counter;
pub mod time_manager;
pub mod search_limit;
