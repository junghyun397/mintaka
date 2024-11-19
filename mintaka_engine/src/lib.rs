#![feature(stdarch_aarch64_prefetch)]
#![feature(adt_const_params)]

pub mod config;
pub mod memo;
pub mod eval;
pub mod search;
pub mod tablebase;
pub mod worker;
pub mod utils;
pub mod protocol;
