use arrayvec::ArrayVec;
use rusty_renju::impl_display_from_debug;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::value::Score;

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct PrincipalVariation {
    pub score: Score,
    pub moves: ArrayVec<Pos, 128>,
}

impl_display_from_debug!(PrincipalVariation);

impl PrincipalVariation {

}
