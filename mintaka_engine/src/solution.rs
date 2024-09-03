use mintaka::notation::pos::Pos;
use std::iter::Map;

pub type Solution = Option<Map<Pos, SolutionNone>>;

pub struct SolutionNone {
    pub solution: Pos,
    pub child: Solution
}
