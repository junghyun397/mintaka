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
pub struct ComplexBound {
    pub node_count: NodeCount,
    pub time_bound: TimeBound,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SearchLimit {
    Nodes(NodeCount),
    Time(TimeBound),
    Complex(ComplexBound),
    Infinite,
}
