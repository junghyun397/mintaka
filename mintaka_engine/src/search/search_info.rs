use std::sync::atomic::{AtomicBool, AtomicUsize};

#[derive(Debug, Default)]
pub struct SearchInfo {
    pub visited_nodes: AtomicUsize,
    pub aborted: AtomicBool,
}
