use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone)]
pub struct BatchCounter<'a> {
    buffer: usize,
    global_counter_in_1k: &'a AtomicUsize,
    local_counter_in_1k: usize,
}

impl<'a> BatchCounter<'a> {

    pub const fn new(global_counter_in_1k: &'a AtomicUsize) -> Self {
        Self {
            buffer: 0,
            global_counter_in_1k,
            local_counter_in_1k: 0,
        }
    }

    pub fn increment_single(&mut self) {
        self.increment_amount(1);
    }

    pub fn increment_pair(&mut self) {
        self.increment_amount(2);
    }

    fn increment_amount(&mut self, amount: usize) {
        self.buffer += amount;
        if self.buffer >= 1000 {
            self.global_counter_in_1k.fetch_add(1, Ordering::Relaxed);
            self.local_counter_in_1k += 1;
            self.buffer -= 1000;
        }
    }

    pub fn count_global_in_1k(&self) -> usize {
        self.global_counter_in_1k.load(Ordering::Relaxed)
    }

    pub fn count_local_total(&self) -> usize {
        self.local_counter_in_1k * 1000 + self.buffer
    }

    pub fn buffer_zero(&self) -> bool {
        self.buffer == 0
    }

}
