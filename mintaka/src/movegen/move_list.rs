use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::value::{Score, Scores};

#[derive(Debug)]
pub struct MoveList {
    moves: [(Pos, Score); pos::BOARD_SIZE],
    top: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl MoveList {

    const EMPTY: Self = Self {
        moves: [(MaybePos::INVALID_POS, -Score::INF); pos::BOARD_SIZE],
        top: 0,
    };

    pub fn push(&mut self, pos: Pos, score: Score) {
        self.moves[self.top] = (pos, score);
        self.top += 1;
    }

    pub fn consume_best(&mut self) -> Option<(Pos, Score)> {
        if self.top == 0 {
            return None;
        }

        let mut best_idx = 0;
        let mut best_score = i16::MIN;

        for (idx, &(_, score)) in self.moves[0 .. self.top].iter().enumerate() {
            if score > best_score {
                best_score = score;
                best_idx = idx;
            }
        }

        self.top -= 1;
        self.moves.swap(best_idx, self.top);

        Some(self.moves[self.top])
    }

}
