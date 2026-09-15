use std::time::Duration;
#[cfg(feature = "typeshare")]
use typeshare::typeshare;
use crate::protocol::nodes::Nodes;
use crate::protocol::time::{TimeUnit, TimeValue};

#[cfg_attr(feature = "typeshare", typeshare::typeshare)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde_with::skip_serializing_none)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Timer {
    pub time_unit: TimeUnit,
    pub total_remaining: Option<TimeValue>,
    pub increment: TimeValue,
    pub turn: Option<TimeValue>,
}

impl Timer {
    pub const fn new(
        time_unit: TimeUnit,
        total_time: Option<TimeValue>,
        increment: TimeValue,
        turn: Option<TimeValue>,
    ) -> Self {
        Self {
            time_unit,
            total_remaining: total_time,
            increment,
            turn,
        }
    }

    pub const fn infinite(time_unit: TimeUnit) -> Self {
        Self {
            time_unit,
            total_remaining: None,
            increment: TimeValue::ZERO,
            turn: None,
        }
    }

    pub const fn fixed_time(turn_time: Duration) -> Self {
        Self {
            time_unit: TimeUnit::Clock,
            total_remaining: None,
            increment: TimeValue::ZERO,
            turn: Some(TimeValue::from_duration(turn_time)),
        }
    }

    pub const fn fixed_nodes(nodes: Nodes) -> Self {
        Self {
            time_unit: TimeUnit::Nodes,
            total_remaining: None,
            increment: TimeValue::ZERO,
            turn: Some(TimeValue::from_nodes(nodes)),
        }
    }

    pub fn consume(&mut self, running_time: TimeValue) {
        if let Some(total_remaining) = &mut self.total_remaining {
            *total_remaining = *total_remaining - running_time;
        }
    }

    pub fn apply_increment(&mut self) {
        if let Some(total_remaining) = &mut self.total_remaining {
            *total_remaining = *total_remaining + self.increment;
        }
    }

    pub fn append(&mut self, additional_time: TimeValue) {
        if let Some(total_remaining) = &mut self.total_remaining {
            *total_remaining = *total_remaining + additional_time;
        }
    }
}
