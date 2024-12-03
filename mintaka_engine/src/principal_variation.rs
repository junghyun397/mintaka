use mintaka::notation::node::Score;
use mintaka::notation::pos::Pos;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PrincipalVariation {
    pub score: Score,
    pub moves: [Pos; 64],
    pub moves_top: usize,
}
