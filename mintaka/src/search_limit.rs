use rusty_renju::notation::value::Depth;
use std::time::Duration;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct NodeCount {
    pub nodes_in_1k: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TimeBound {
    pub duration: Duration,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SearchLimit {
    Depth(Depth),
    Nodes(NodeCount),
    Time(TimeBound),
    Infinite,
}
