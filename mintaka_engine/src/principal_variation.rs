use mintaka::notation::node::Score;
use mintaka::notation::pos::Pos;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PrincipalVariation {
    pub score: Score,
    pub moves: [Pos; 64],
    pub moves_top: usize,
}

impl Default for PrincipalVariation {

    fn default() -> Self {
        Self::EMPTY
    }

}

impl PrincipalVariation {

    const EMPTY: Self = Self {
        score: 0,
        moves: [Pos::INVALID; 64],
        moves_top: 0,
    };

}
