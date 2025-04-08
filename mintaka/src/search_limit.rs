use std::time::Duration;

#[derive(Debug, Clone)]
pub enum SearchLimit {
    Time { finish_at: Duration },
    Nodes { in_1k: usize },
}
