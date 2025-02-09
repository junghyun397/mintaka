use std::sync::atomic::{AtomicUsize, Ordering};

pub struct BatchCounter<'a> {
    buffer: usize,
    global_counter_in_1k: &'a AtomicUsize,
    pub local_counter_in_1k: usize,
}

impl<'a> BatchCounter<'a> {

    pub const fn new(global_counter_in_1k: &'a AtomicUsize) -> Self {
        Self {
            buffer: 0,
            global_counter_in_1k,
            local_counter_in_1k: 0,
        }
    }

    pub fn add_single_mut(&mut self) {
        self.add_amount_mut(1);
    }

    pub fn add_pair_mut(&mut self) {
        self.add_amount_mut(2);
    }

    fn add_amount_mut(&mut self, amount: usize) {
        self.buffer += amount;
        if self.buffer >= 1024 {
            self.global_counter_in_1k.fetch_add(1, Ordering::Relaxed);
            self.local_counter_in_1k += 1;
            self.buffer = 0;
        }
    }

    pub fn clear_local(&mut self) {
        self.buffer = 0;
        self.local_counter_in_1k = 0;
    }

    pub fn count_local_total(&self) -> usize {
        self.local_counter_in_1k * 1000 + self.buffer
    }

}
