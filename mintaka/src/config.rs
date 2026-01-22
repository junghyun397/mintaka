use std::cmp::Ordering;
use crate::time::Timer;
use crate::value::{Depth, Depths};
use rusty_renju::history;
use rusty_renju::notation::pos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::utils::byte_size::ByteSize;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::time::Duration;
#[cfg(feature = "typeshare")]
use typeshare::typeshare;

#[cfg_attr(feature = "typeshare", typeshare(serialized_as = "String"))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum SearchObjective {
    #[default] Best = 0,
    Zeroing = 1,
    Pondering = 2
}

#[cfg_attr(feature = "typeshare", typeshare)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde_with::skip_serializing_none)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Config {
    pub rule_kind: RuleKind,
    pub draw_condition: u32,

    pub max_nodes_in_1k: Option<u32>,
    pub max_depth: Option<Depth>,
    pub max_vcf_depth: Option<Depth>,

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
            draw_condition: pos::BOARD_SIZE as u32,
            max_nodes_in_1k: None,
            max_depth: None,
            max_vcf_depth: None,
            tt_size: ByteSize::from_mib(512),
            workers: 1,
            pondering: false,
            dynamic_time: false,
            initial_timer: Timer::default(),
            spawn_depth_specialist: false,
        }
    }
}

impl PartialOrd<Self> for Config {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Config {
    fn cmp(&self, other: &Self) -> Ordering {
        (
            self.max_nodes_in_1k,
            self.max_depth,
            self.max_vcf_depth,
            self.tt_size,
            self.workers,
            self.pondering,
            self.initial_timer,
        )
            .cmp(&(
                other.max_nodes_in_1k,
                other.max_depth,
                other.max_vcf_depth,
                other.tt_size,
                other.workers,
                other.pondering,
                other.initial_timer,
            ))
    }
}

impl Config {

    pub const UNLIMITED_CONFIG: Self = Self {
        rule_kind: RuleKind::Renju,
        draw_condition: pos::BOARD_SIZE as u32,
        max_nodes_in_1k: None,
        max_depth: None,
        max_vcf_depth: None,
        tt_size: ByteSize::from_mib(1024 * 1024 * 1024),
        workers: 2048,
        pondering: true,
        dynamic_time: true,
        initial_timer: Timer {
            total_remaining: None,
            increment: Duration::from_secs(u32::MAX as u64),
            turn: None,
        },
        spawn_depth_specialist: true,
    };

    pub fn max_depth(&self) -> Depth {
        self.max_depth.unwrap_or(Depth::PLY_LIMIT)
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
        if self.draw_condition > history::MAX_HISTORY_SIZE as u32 {
            Err(ConfigValidationError::DrawConditionDeeperThenMaxHistory)
        } else if self.max_depth > Some(Depth::PLY_LIMIT) {
            Err(ConfigValidationError::DepthDeeperThanMaxPly)
        } else if self.max_vcf_depth > Some(Depth::PLY_LIMIT) {
            Err(ConfigValidationError::VCFDepthDeeperThanMaxPly)
        } else {
            Ok(self)
        }
    }

}
