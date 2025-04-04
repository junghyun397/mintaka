use rusty_renju::bitfield::Bitfield;
use rusty_renju::cartesian_to_index;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use rusty_renju::pattern::Pattern;

#[derive(Debug, Copy, Clone)]
pub struct MoveScores {
    pub scores: [u8; pos::BOARD_SIZE],
}

impl Default for MoveScores {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl From<Bitfield> for MoveScores {
    fn from(value: Bitfield) -> Self {
        value.iter_hot_pos()
            .fold(Self::default(), |mut acc, pos| {
                acc.add_neighborhood_score(pos);
                acc
            })
    }
}

impl MoveScores {

    pub const EMPTY: MoveScores = MoveScores {
        scores: [0; pos::BOARD_SIZE],
    };

    pub fn update_pattern_score(&mut self, pos: Pos, pattern: Pattern) {
        todo!()
    }

    pub fn add_neighborhood_score(&mut self, pos: Pos) {
        for row in
            pos.row().saturating_sub(2) .. (pos.row() + 2).min(pos::BOARD_WIDTH - 1)
        {
            for col in
                pos.col().saturating_sub(2) .. (pos.col() + 2).min(pos::BOARD_WIDTH - 1)
            {
                let idx = cartesian_to_index!(row, col) as usize;
                self.scores[idx] = self.scores[idx].saturating_add(1);
            }
        }
    }

    pub fn remove_neighborhood_score(&mut self, pos: Pos) {
        for row in
            pos.row().saturating_sub(2) .. (pos.row() + 2).min(pos::BOARD_WIDTH - 1)
        {
            for col in
                pos.col().saturating_sub(2) .. (pos.col() + 2).min(pos::BOARD_WIDTH - 1)
            {
                let idx = cartesian_to_index!(row, col) as usize;
                self.scores[idx] = self.scores[idx].saturating_sub(1);
            }
        }
    }

}
