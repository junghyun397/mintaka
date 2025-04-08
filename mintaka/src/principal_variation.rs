use arrayvec::ArrayVec;
use rusty_renju::impl_display_from_debug;
use rusty_renju::notation::pos::MaybePos;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PrincipalVariation {
    pub line: ArrayVec<MaybePos, 128>,
}

impl_display_from_debug!(PrincipalVariation);

impl PrincipalVariation {

    pub const fn new_const() -> Self {
        Self {
            line: ArrayVec::new_const(),
        }
    }

    pub fn clear(&mut self) {
        self.line.clear();
    }

    pub fn load(&mut self, head: MaybePos, rest: Self) {
        self.line.clear();
        self.line.push(head);
        self.line.extend(rest.line);
    }

}
