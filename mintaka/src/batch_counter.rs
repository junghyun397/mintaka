use std::sync::atomic::{AtomicU32, Ordering};
use crate::protocol::nodes::Nodes;

#[derive(Clone)]
pub struct BatchCounter<'a> {
    buffer: u32,
    global_counter_in_1k: &'a AtomicU32,
    local_counter_in_1k: u32,
}

impl<'a> BatchCounter<'a> {
    pub const fn new(global_counter_in_1k: &'a AtomicU32) -> Self {
        Self {
            buffer: 0,
            global_counter_in_1k,
            local_counter_in_1k: 0,
        }
    }

    pub fn increment(&mut self) {
        self.buffer += 1;
        if self.buffer >= 1000 {
            self.global_counter_in_1k.fetch_add(1, Ordering::Relaxed);
            self.local_counter_in_1k += 1;
            self.buffer -= 1000;
        }
    }

    pub fn count_global(&self) -> Nodes {
        Nodes::from_in_1k(self.global_counter_in_1k.load(Ordering::Relaxed))
    }

    pub fn count_local(&self) -> Nodes {
        Nodes::from_in_1k(self.local_counter_in_1k + (self.buffer / 1000))
    }

    pub fn buffer_zero(&self) -> bool {
        self.buffer == 0
    }
}
