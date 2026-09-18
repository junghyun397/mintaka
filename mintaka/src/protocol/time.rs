use std::fmt::Display;
use std::ops::{Add, Div, Mul, Sub};
use std::str::FromStr;
use std::time::Duration;
#[cfg(feature = "serde")]
use serde_with::{As, DisplayFromStr, IfIsHumanReadable};
#[cfg(feature = "typeshare")]
use typeshare::typeshare;
use crate::protocol::nodes::Nodes;

#[cfg_attr(feature = "typeshare", typeshare(serialized_as = "String"))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TimeUnit {
    Clock,
    Nodes
}

impl Display for TimeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeUnit::Clock => write!(f, "Clock"),
            TimeUnit::Nodes => write!(f, "Nodes"),
        }
    }
}

impl FromStr for TimeUnit {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Clock" => Ok(TimeUnit::Clock),
            "Nodes" => Ok(TimeUnit::Nodes),
            _ => Err("invalid time unit"),
        }
    }
}

#[cfg_attr(feature = "typeshare", typeshare(serialized_as = "String"))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct TimeValue(
    #[cfg_attr(feature = "serde", serde(with = "As::<IfIsHumanReadable<DisplayFromStr>>"))]
    u128
);

impl TimeValue {
    pub const ZERO: Self = Self::new(0);

    pub const INFINITE: Self = Self::new(u128::MAX);

    pub const fn new(value: u128) -> Self {
        Self(value)
    }

    pub const fn from_value(value: u64, unit: TimeUnit) -> Self {
        match unit {
            TimeUnit::Clock => Self::from_duration(Duration::from_millis(value)),
            TimeUnit::Nodes => Self::from_nodes(Nodes::from_in_1k(value as u32)),
        }
    }

    pub const fn from_duration(duration: Duration) -> Self {
        Self::new(duration.as_nanos())
    }

    pub const fn to_duration(&self) -> Duration {
        Duration::from_nanos(self.0 as u64)
    }

    pub const fn from_nodes(nodes: Nodes) -> Self {
        Self::new(nodes.in_1k as u128 * 1_000)
    }
    
    pub const fn to_nodes(&self) -> Nodes {
        Nodes::from_in_1k((self.0 / 1_000) as u32)
    }

    pub fn multiply_clamp(&self, factor: f64, max: Self) -> Self {
        Self::new(((self.0 as f64 * factor) as u128).min(max.0))
    }
}

impl From<u128> for TimeValue {
    fn from(value: u128) -> Self {
        Self::new(value)
    }
}

impl From<TimeValue> for u128 {
    fn from(time_value: TimeValue) -> Self {
        time_value.0
    }
}

impl Add for TimeValue {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.0.saturating_add(rhs.0))
    }
}

impl Sub for TimeValue {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.0.saturating_sub(rhs.0))
    }
}

impl Mul<u32> for TimeValue {
    type Output = Self;
    fn mul(self, rhs: u32) -> Self::Output {
        Self::new(self.0.saturating_mul(rhs as u128))
    }
}

impl Div<u32> for TimeValue {
    type Output = Self;
    fn div(self, rhs: u32) -> Self::Output {
        Self::new(self.0 / rhs as u128)
    }
}
