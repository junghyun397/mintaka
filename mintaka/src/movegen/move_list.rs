use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::value::Score;

#[derive(Debug, Copy, Clone)]
pub struct MoveEntry {
    pub pos: Pos,
    pub score: Score,
}

#[derive(Debug)]
pub struct MoveList {
    moves: [MoveEntry; pos::BOARD_SIZE],
    top: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl MoveList {

    const EMPTY: Self = unsafe { std::mem::zeroed() };

    pub fn push(&mut self, pos: Pos, score: Score) {
        self.moves[self.top] = MoveEntry { pos, score };
        self.top += 1;
    }

    pub fn consume_best(&mut self) -> Option<MoveEntry> {
        if self.top == 0 {
            return None;
        }

        let mut best_idx = 0;
        let mut best_score = Score::MIN;

        for (idx, &MoveEntry { score, .. }) in self.moves[0 .. self.top].iter().enumerate() {
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
