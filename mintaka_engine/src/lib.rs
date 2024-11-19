#![allow(incomplete_features)]
#![feature(portable_simd)]
#![feature(adt_const_params)]

#![cfg_attr(target_arch = "aarch64", feature(stdarch_aarch64_prefetch))]

pub mod config;
pub mod memo;
pub mod eval;
pub mod search;
pub mod tablebase;
pub mod worker;
pub mod utils;
pub mod protocol;
