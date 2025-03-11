use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TimeManager {
    pub total_remaining: Duration,
    pub increment: Duration,
    pub overhead: Duration,
}

impl Default for TimeManager {

    fn default() -> Self {
        Self {
            total_remaining: Duration::from_secs(60 * 30),
            increment: Duration::from_secs(30),
            overhead: Duration::from_secs(30),
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
            overhead,
        }
    }

    pub fn next_running_time(&self) -> Duration {
        self.total_remaining / 20 + self.increment
    }

    pub fn consume(self, running_time: Duration) -> Self {
        Self {
            total_remaining: self.total_remaining - running_time,
            increment: self.increment,
            overhead: self.overhead,
        }
    }

    pub fn append(self, additional_time: Duration) -> Self {
        Self {
            total_remaining: self.total_remaining + additional_time,
            increment: self.increment,
            overhead: self.overhead,
        }
    }

    pub fn subtract(self, revoked_time: Duration) -> Self {
        Self {
            total_remaining: self.total_remaining - revoked_time,
            increment: self.increment,
            overhead: self.overhead,
        }
    }

}
