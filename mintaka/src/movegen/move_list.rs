use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos};

#[derive(Debug)]
pub struct MoveList {
    moves: [(Pos, i32); pos::BOARD_SIZE],
    top: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl MoveList {

    const EMPTY: Self = Self {
        moves: [(MaybePos::NONE.unwrap(), i32::MIN); pos::BOARD_SIZE],
        top: 0,
    };

    pub fn push(&mut self, pos: Pos, score: i32) {
        self.moves[self.top] = (pos, score);
        self.top += 1;
    }

    pub fn len(&self) -> usize {
        self.top
    }

    pub fn is_empty(&self) -> bool {
        self.top == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = &(Pos, i32)> {
        self.moves[..self.top].iter()
    }

    pub fn consume(&mut self, idx: usize) -> (Pos, i32) {
        self.top -= 1;
        self.moves.swap(idx, self.top);
        self.moves[self.top]
    }

}
