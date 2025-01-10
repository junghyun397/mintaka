use std::sync::atomic::{AtomicBool, AtomicUsize};

#[derive(Debug, Default)]
pub struct SearchInfo {
    pub visited_nodes_in_kilos: AtomicUsize,
    pub aborted: AtomicBool,
}
