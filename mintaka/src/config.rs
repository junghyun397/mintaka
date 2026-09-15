use crate::protocol::timer::Timer;
use crate::utils::depth::Depth;
use rusty_renju::utils::byte_size::ByteSize;
use std::fmt::Display;
use crate::protocol::time::TimeUnit;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum SearchObjective {
    #[default] Best = 0,
    Zeroing = 1,
    Pondering = 2
}

#[cfg_attr(feature = "typeshare", typeshare::typeshare)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde_with::skip_serializing_none)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Config {
    pub draw_condition: Option<u32>,

    pub max_depth: Option<Depth>,
    pub max_quiescence_depth: Option<Depth>,

    pub tt_size: ByteSize,
    pub workers: u32,
    pub pondering: bool,

    pub initial_timer: Timer,

    pub spawn_depth_specialist: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            draw_condition: None,
            max_depth: None,
            max_quiescence_depth: None,
            tt_size: ByteSize::from_mib(128),
            workers: 1,
            pondering: false,
            initial_timer: Timer::infinite(TimeUnit::Clock),
            spawn_depth_specialist: false,
        }
    }
}

impl Config {
    pub const UNLIMITED_CONFIG: Self = Self {
        draw_condition: None,
        max_depth: None,
        max_quiescence_depth: None,
        tt_size: ByteSize::from_mib(1024 * 1024 * 1024),
        workers: 2048,
        pondering: true,
        initial_timer: Timer::infinite(TimeUnit::Clock),
        spawn_depth_specialist: true,
    };

    pub fn max_depth(&self) -> Depth {
        self.max_depth.unwrap_or(Depth::PLY_LIMIT)
    }
}

#[derive(Debug)]
pub enum ConfigValidationError {
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
        if self.max_depth > Some(Depth::PLY_LIMIT) {
            Err(ConfigValidationError::DepthDeeperThanMaxPly)
        } else if self.max_quiescence_depth > Some(Depth::PLY_LIMIT) {
            Err(ConfigValidationError::VCFDepthDeeperThanMaxPly)
        } else {
            Ok(self)
        }
    }
}
