use crate::utils::time_manager::TimeManager;
use crate::value;
use rusty_renju::notation::pos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::utils::byte_size::ByteSize;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum SearchObjective {
    #[default] Best,
    Zeroing,
    Pondering
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Config {
    pub rule_kind: RuleKind,
    pub draw_condition: usize,

    pub search_objective: SearchObjective,

    pub max_nodes_in_1k: Option<usize>,
    pub max_depth: usize,
    pub max_vcf_depth: usize,

    pub tt_size: ByteSize,
    pub workers: NonZeroU32,

    pub initial_time_manager: Option<TimeManager>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            rule_kind: RuleKind::Renju,
            draw_condition: pos::BOARD_SIZE,
            search_objective: SearchObjective::default(),
            max_nodes_in_1k: None,
            max_depth: value::MAX_PLY,
            max_vcf_depth: pos::BOARD_SIZE - 5,
            tt_size: ByteSize::from_mib(16),
            workers: NonZeroU32::new(1).unwrap(),
            initial_time_manager: None,
        }
    }
}
