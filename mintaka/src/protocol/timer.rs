use std::time::Duration;

#[cfg_attr(feature = "typeshare", typeshare::typeshare)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde_with::skip_serializing_none)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Timer {
    pub total_remaining: Option<Duration>,
    pub increment: Duration,
    pub turn: Option<Duration>,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            total_remaining: Some(Duration::from_secs(60 * 5)),
            increment: Duration::from_secs(0),
            turn: Some(Duration::from_secs(30)),
        }
    }
}

impl Timer {
    pub const INFINITE: Self = Self::new(None, Duration::ZERO, None);

    pub const fn new(
        total_time: Option<Duration>,
        increment: Duration,
        turn: Option<Duration>,
    ) -> Self {
        Self {
            total_remaining: total_time,
            increment,
            turn,
        }
    }

    pub fn consume(&mut self, running_time: Duration) {
        if let Some(total_remaining) = &mut self.total_remaining {
            *total_remaining = total_remaining.saturating_sub(running_time);
        }
    }

    pub fn apply_increment(&mut self) {
        if let Some(total_remaining) = &mut self.total_remaining {
            *total_remaining += self.increment;
        }
    }

    pub fn append(&mut self, additional_time: Duration) {
        if let Some(total_remaining) = &mut self.total_remaining {
            *total_remaining += additional_time;
        }
    }
}
