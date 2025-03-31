use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos};

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
        moves: [(MaybePos::NONE.unwrap(), 0); pos::BOARD_SIZE],
        top: 0,
    };

    pub fn push(&mut self, pos: Pos, score: i32) {
        self.moves[self.top] = (pos, score);
        self.top += 1;
    }

    pub fn len(&self) -> usize {
        self.top
    }

}
