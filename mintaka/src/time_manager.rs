use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct TimeManager {
    pub total_remaining: Duration,
    pub increment: Duration,
    pub turn: Duration,
}

impl Default for TimeManager {

    fn default() -> Self {
        Self {
            total_remaining: Duration::from_secs(60 * 10),
            increment: Duration::from_secs(30),
            turn: Duration::from_secs(60),
        }
    }

}

impl TimeManager {

    pub fn new(
        total_time: Duration,
        increment: Duration,
        overhead: Duration,
    ) -> Self {
        Self {
            total_remaining: total_time,
            increment,
            turn: overhead,
        }
    }

    pub fn next_running_time(&self) -> Duration {
        (self.total_remaining / 20 + self.increment).min(self.turn)
    }

    pub fn consume(&mut self, running_time: Duration) {
        self.total_remaining = self.total_remaining.saturating_sub(running_time);
    }

    pub fn append(&mut self, additional_time: Duration) {
        self.total_remaining += additional_time;
    }

}
