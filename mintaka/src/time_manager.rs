use rusty_renju::notation::pos::MaybePos;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Timer {
    pub total_remaining: Duration,
    pub increment: Duration,
    pub turn: Duration,
}

impl Default for Timer {

    fn default() -> Self {
        Self {
            total_remaining: Duration::from_secs(60 * 5),
            increment: Duration::from_secs(0),
            turn: Duration::from_secs(30),
        }
    }

}

impl Timer {

    pub const INFINITE: Self = Self::new(Duration::MAX, Duration::ZERO, Duration::MAX);

    pub const fn new(
        total_time: Duration,
        increment: Duration,
        turn: Duration,
    ) -> Self {
        Self {
            total_remaining: total_time,
            increment,
            turn,
        }
    }

    pub fn is_infinite(&self) -> bool {
        self.total_remaining == Duration::MAX && self.turn == Duration::MAX
    }

    pub fn consume(&mut self, running_time: Duration) {
        self.total_remaining = self.total_remaining.saturating_sub(running_time);
    }

    pub fn apply_increment(&mut self) {
        self.total_remaining += self.increment;
    }

    pub fn append(&mut self, additional_time: Duration) {
        self.total_remaining += additional_time;
    }

}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
            Some(self.timer.total_remaining / 20 + self.timer.increment)
        } else {
            Some(self.timer.turn)
        }
    }

}
