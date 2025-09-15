use rusty_renju::chebyshev_distance;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::score::Score;

pub trait EndgameAccumulator {

    const DISTANCE_WINDOW: isize;

    const ZERO: Self;

    fn unit(pos: Pos, score: Score) -> Self;

    fn append_pos(self, defend: Pos, threat: Pos) -> Self;

    fn score(&self) -> Score;

}

pub type SequenceEndgameAccumulator = Option<Vec<MaybePos>>;

impl EndgameAccumulator for SequenceEndgameAccumulator {

    const DISTANCE_WINDOW: isize = 5;

    const ZERO: Self = None;

    fn unit(pos: Pos, _score: Score) -> Self {
        Some(vec![pos.into()])
    }

    fn append_pos(self, defend: Pos, four: Pos) -> Self {
        self.map(|mut sequence| {
            sequence.push(defend.into());
            sequence.push(four.into());
            sequence
        })
    }

    fn score(&self) -> Score {
        0
    }

}

impl EndgameAccumulator for Score {

    const DISTANCE_WINDOW: isize = 5;

    const ZERO: Self = 0;

    fn unit(_pos: Pos, score: Score) -> Self {
        score
    }

    fn append_pos(self, _defend: Pos, _four: Pos) -> Self {
        self
    }

    fn score(&self) -> Score {
        *self
    }

}

pub const ENDGAME_MAX_MOVES: usize = 31;

#[derive(Debug, Copy, Clone)]
pub struct EndgameMovesUnchecked {
    pub moves: [MaybePos; ENDGAME_MAX_MOVES],
    pub top: u8,
}

impl EndgameMovesUnchecked {

    pub fn unit(pos: Pos) -> Self {
        Self {
            moves: {
                const EMPTY_MOVES: [MaybePos; ENDGAME_MAX_MOVES] = [MaybePos::NONE; ENDGAME_MAX_MOVES];

                let mut new_moves = EMPTY_MOVES;
                new_moves[0] = pos.into();
                new_moves
            },
            top: 1,
        }
    }

    pub fn init(&mut self) {
        self.top = 0;
    }

    pub fn next(&mut self) -> Option<Pos> {
        if self.top == 32 {
            return None;
        }

        let next_move = self.moves[self.top as usize].into();
        self.top += 1;
        next_move
    }

    pub fn sort_moves(&mut self, ref_pos: Pos) {
        let ref_row = ref_pos.row() as i16;
        let ref_col = ref_pos.col() as i16;

        self.moves[..self.top as usize].sort_by_key(|&pos| {
            chebyshev_distance!(ref_row, ref_col, pos.unwrap_unchecked().row() as i16, pos.unwrap_unchecked().col() as i16)
        });
    }

    pub fn is_empty(&self) -> bool {
        self.top == 0
    }

}

#[derive(Copy, Clone)]
pub struct EndgameFrame {
    pub moves: EndgameMovesUnchecked,
    pub alpha: Score,
    pub four_pos: Pos,
    pub defend_pos: Pos,
}
