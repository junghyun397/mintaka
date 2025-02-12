use rusty_renju::notation::value::Depth;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct NodeCount {
    pub nodes_in_1k: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TimeBound {
    pub epoch_time_lower_64: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SearchLimit {
    Depth(Depth),
    Nodes(NodeCount),
    Time(TimeBound),
    Infinite,
}
