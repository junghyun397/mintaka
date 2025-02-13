use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::value::Score;
use smallvec::SmallVec;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PrincipalVariation {
    pub score: Score,
    pub moves: SmallVec<[Pos; 128]>,
}

impl Default for PrincipalVariation {

    fn default() -> Self {
        Self {
            score: 0,
            moves: SmallVec::new(),
        }
    }

}

impl PrincipalVariation {

}
