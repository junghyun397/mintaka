use rusty_renju::notation::pos::MaybePos;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::time::Duration;
#[allow(unused_imports)]
use rusty_renju::utils::lang::DurationSchema;

#[cfg_attr(feature = "typeshare", typeshare::typeshare)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Timer {
    #[cfg_attr(feature = "typeshare", typeshare(serialized_as = "Option<DurationSchema>"))]
    pub total_remaining: Option<Duration>,
    #[cfg_attr(feature = "typeshare", typeshare(serialized_as = "DurationSchema"))]
    pub increment: Duration,
    #[cfg_attr(feature = "typeshare", typeshare(serialized_as = "Option<DurationSchema>"))]
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

    pub fn is_infinite(&self) -> bool {
        self.total_remaining.is_none() && self.turn.is_none()
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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct TimeManager {
    pub timer: Timer,
    pub time_history: Vec<(MaybePos, Duration)>,
    pub dynamic_time: bool,
    stability: usize,
    pv_stability: usize,
    fail_low_count: usize,
}

impl From<Timer> for TimeManager {
    fn from(timer: Timer) -> Self {
        Self {
            timer,
            time_history: Vec::new(),
            dynamic_time: false,
            stability: 0,
            pv_stability: 0,
            fail_low_count: 0,
        }
    }
}

impl TimeManager {

    pub fn new(timer: Timer) -> Self {
        Self {
            timer,
            time_history: Vec::new(),
            dynamic_time: false,
            stability: 0,
            pv_stability: 0,
            fail_low_count: 0,
        }
    }

    pub fn next_running_time(&self) -> Option<Duration> {
        if self.timer.is_infinite() {
            None
        } else if self.dynamic_time {
            self.timer.total_remaining
                .map(|total_remaining| total_remaining / 20 + self.timer.increment)
                .or(self.timer.turn)
        } else {
            self.timer.turn
        }
    }

}
