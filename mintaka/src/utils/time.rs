pub trait MonotonicClock : Send + Sync + Copy {

    fn now() -> Self;

    fn elapsed_since(&self, start: Self) -> std::time::Duration;

    fn elapsed(&self) -> std::time::Duration {
        Self::now().elapsed_since(*self)
    }

}

impl MonotonicClock for std::time::Instant {
    fn now() -> Self {
        Self::now()
    }

    fn elapsed_since(&self, start: Self) -> std::time::Duration {
        self.duration_since(start)
    }
}
