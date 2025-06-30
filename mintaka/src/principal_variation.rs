use crate::value::MAX_PLY;
use rusty_renju::impl_display_from_debug;
use rusty_renju::notation::pos::MaybePos;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PrincipalVariation {
    pub line: [MaybePos; MAX_PLY],
    pub top: usize,
}

impl_display_from_debug!(PrincipalVariation);

impl PrincipalVariation {

    pub const fn new_const() -> Self {
        Self {
            line: [MaybePos::NONE; MAX_PLY],
            top: 0,
        }
    }

    pub fn head(&self) -> MaybePos {
        self.line[0]
    }

    pub fn clear(&mut self) {
        self.top = 0;
    }

    pub fn load(&mut self, head: MaybePos, rest: &Self) {
        self.clear();
        self.line[0] = head;
        self.top = rest.top + 1;
        self.line[1 .. self.top].copy_from_slice(&rest.line[0 .. rest.top]);
    }

}
