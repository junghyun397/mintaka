use crate::time_manager::Timer;
use crate::value::{Depth, Depths};
use rusty_renju::history;
use rusty_renju::notation::pos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::utils::byte_size::ByteSize;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use typeshare::typeshare;

#[typeshare(serialized_as = "String")] // using string to avoid ts enum
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum SearchObjective {
    #[default] Best = 0,
    Zeroing = 1,
    Pondering = 2
}

#[typeshare::typeshare]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Config {
    pub rule_kind: RuleKind,
    #[typeshare(serialized_as = "number")]
    pub draw_condition: u64,

    #[typeshare(serialized_as = "number")]
    pub max_nodes_in_1k: Option<u64>,
    pub max_depth: Depth,
    pub max_vcf_depth: Depth,

    pub tt_size: ByteSize,
    pub workers: u32,
    pub pondering: bool,

    pub dynamic_time: bool,
    pub initial_timer: Timer,

    pub spawn_depth_specialist: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            rule_kind: RuleKind::Renju,
            draw_condition: pos::BOARD_SIZE as u64,
            max_nodes_in_1k: None,
            max_depth: Depth::PLY_LIMIT,
            max_vcf_depth: 24,
            tt_size: ByteSize::from_mib(512),
            workers: 1,
            pondering: false,
            dynamic_time: false,
            initial_timer: Timer::default(),
            spawn_depth_specialist: false,
        }
    }
}

#[derive(Debug)]
pub enum ConfigValidationError {
    DrawConditionDeeperThenMaxHistory,
    DepthDeeperThanMaxPly,
    VCFDepthDeeperThanMaxPly,
}

impl Display for ConfigValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for ConfigValidationError {}

impl Config {

    pub fn validate(self) -> Result<Self, ConfigValidationError> {
        if self.draw_condition > history::MAX_HISTORY_SIZE as u64 {
            Err(ConfigValidationError::DrawConditionDeeperThenMaxHistory)
        } else if self.max_depth > Depth::PLY_LIMIT {
            Err(ConfigValidationError::DepthDeeperThanMaxPly)
        } else if self.max_vcf_depth > Depth::PLY_LIMIT {
            Err(ConfigValidationError::VCFDepthDeeperThanMaxPly)
        } else {
            Ok(self)
        }
    }

}
