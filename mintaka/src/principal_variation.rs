use arrayvec::ArrayVec;
use rusty_renju::impl_display_from_debug;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct PrincipalVariation {
    pub line: ArrayVec<Pos, { pos::BOARD_SIZE }>,
}

impl_display_from_debug!(PrincipalVariation);

impl PrincipalVariation {

    fn load(&mut self, pos: Pos, other: Self) {
        self.line.clear();
        self.line.push(pos);
        self.line.extend(other.line);
    }

}
