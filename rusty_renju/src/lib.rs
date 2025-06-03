#![allow(incomplete_features)]
#![feature(adt_const_params)]
#![feature(portable_simd)]
#![feature(avx512_target_feature)]

#![cfg_attr(target_arch = "aarch64", feature(stdarch_aarch64_prefetch))]

extern crate core;

pub mod board;
pub mod board_io;
pub mod slice;
pub mod memo;
pub mod notation;
pub mod utils;
pub mod opening;
pub mod pattern;
pub mod slice_pattern;
pub mod board_iter;
pub mod bitfield;
pub mod history;
pub mod slice_pattern_count;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
