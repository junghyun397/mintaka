use crate::time_manager::TimeManager;
use crate::value;
use rusty_renju::history;
use rusty_renju::notation::pos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::utils::byte_size::ByteSize;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Display;

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
    pub workers: u32,

    pub initial_time_manager: Option<TimeManager>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            rule_kind: RuleKind::Renju,
            draw_condition: pos::BOARD_SIZE,
            search_objective: SearchObjective::default(),
            max_nodes_in_1k: None,
            max_depth: 5,
            max_vcf_depth: 24,
            tt_size: ByteSize::from_mib(128),
            workers: 1,
            initial_time_manager: None,
        }
    }
}

#[derive(Debug)]
pub enum ConfigValidationError {
    DrawConditionDeeperThenMaxHistory,
    DepthDeeperThanMaxPly,
}

impl Display for ConfigValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ConfigValidationError {}

impl Config {

    pub fn validate(self) -> Result<Self, ConfigValidationError> {
        if self.draw_condition > history::MAX_HISTORY_SIZE {
            Err(ConfigValidationError::DrawConditionDeeperThenMaxHistory)
        } else if self.max_depth > value::MAX_PLY {
            Err(ConfigValidationError::DepthDeeperThanMaxPly)
        } else {
            Ok(self)
        }
    }

}
