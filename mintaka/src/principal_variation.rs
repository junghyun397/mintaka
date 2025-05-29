use crate::parameters::MAX_PLY;
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

    pub fn push(&mut self, pos: MaybePos) {
        self.line[self.top] = pos;
        self.top += 1;
    }

    fn extend(&mut self, other: Self) {
        self.line[self.top .. self.top + other.top]
            .copy_from_slice(&other.line[..other.top]);
        self.top += other.top;
    }

    pub fn clear(&mut self) {
        self.top = 0;
    }

    pub fn load(&mut self, head: MaybePos, rest: Self) {
        self.clear();
        self.push(head);
        self.extend(rest);
    }

}
