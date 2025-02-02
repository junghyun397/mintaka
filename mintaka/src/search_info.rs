use std::sync::atomic::AtomicUsize;

#[derive(Debug, Default)]
pub struct SearchInfo {
    pub visited_nodes_in_kilos: AtomicUsize,
}
