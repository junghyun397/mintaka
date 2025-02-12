use std::sync::atomic::AtomicBool;
use std::time::Instant;

pub struct TimeManager {
    instant: Instant,
    aborted: AtomicBool,
}
