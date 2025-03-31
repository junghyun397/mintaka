use crate::notation::pos;
use crate::notation::pos::{MaybePos, Pos};

#[derive(Debug, Copy, Clone)]
pub struct History {
    pub entries: [MaybePos; pos::BOARD_SIZE],
    top: usize
}

impl Default for History {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl History {

    const EMPTY: Self = Self {
        entries: [MaybePos::NONE; pos::BOARD_SIZE],
        top: 0,
    };

    pub fn len(&self) -> usize {
        self.top
    }

    pub fn push(&mut self, pos: MaybePos) {
        self.entries[self.top] = pos;
        self.top += 1;
    }

    pub fn pop(&mut self) -> Option<MaybePos> {
        if self.top == 0 {
            return None;
        }

        self.top -= 1;
        let pos = self.entries[self.top];
        Some(pos)
    }

    pub fn is_empty(&self) -> bool {
        self.top == 0
    }

    pub fn action_mut(&mut self, action: MaybePos) {
        self.top += 1;
        self.entries[self.top] = action;
    }

    pub fn play_mut(&mut self, pos: Pos) {
        self.action_mut(pos.into())
    }

    pub fn pass_mut(&mut self) {
        self.top += 1;
        self.entries[self.top] = MaybePos::NONE;
    }

    pub fn iter(&self) -> impl Iterator<Item = &MaybePos> {
        self.entries[..self.top].iter()
    }

    pub fn recent_move(&self) -> MaybePos {
        self.entries[self.top]
    }

    pub fn recent_move_pair(&self) -> [Option<MaybePos>; 2] {
        match self.len() {
            0 => [None, None],
            1 => [Some(self.entries[1]), None],
            _ => [Some(self.entries[self.len() - 2]), Some(self.entries[self.len() - 1])]
        }
    }

}
