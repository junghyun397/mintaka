use crate::utils::batch_counter::BatchCounter;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub struct ThreadData<'a> {
    aborted: &'a AtomicBool,
    pub batch_counter: BatchCounter<'a>,
}

impl<'a> ThreadData<'a> {

    pub const fn new(aborted: &'a AtomicBool, global_counter: &'a AtomicUsize) -> Self {
        Self {
            aborted,
            batch_counter: BatchCounter::new(global_counter),
        }
    }

    fn is_aborted(&self) -> bool {
        self.aborted.load(Ordering::Relaxed)
    }

}
