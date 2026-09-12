use rusty_renju::notation::pos::{self, Pos};
use rusty_renju::utils::empty::Empty;

#[derive(Debug, Copy, Clone)]
pub struct MainMoveEntry {
    pub pos: Pos,
    pub score: i16,
    pub history_score: Option<i16>,
    pub lp_quiet: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct EndgameMoveEntry {
    pub pos: Pos,
    pub score: i16,
}

pub trait MoveEntry: Copy {
    fn ordering_score(&self) -> i16;
}

impl MoveEntry for MainMoveEntry {
    fn ordering_score(&self) -> i16 {
        self.score
    }
}

impl MoveEntry for EndgameMoveEntry {
    fn ordering_score(&self) -> i16 {
        self.score
    }
}

pub type MainMoveList = MoveList<MainMoveEntry, 192>;

pub type EndgameMoveList = MoveList<EndgameMoveEntry, { pos::BOARD_SIZE }>;

#[derive(Debug)]
pub struct MoveList<E: MoveEntry, const N: usize> {
    moves: [E; N],
    top: usize,
}

impl<E: MoveEntry, const N: usize> Empty for MoveList<E, N> {
    fn empty() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl<E: MoveEntry, const N: usize> MoveList<E, N> {
    pub fn is_empty(&self) -> bool {
        self.top == 0
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut E> {
        self.moves[..self.top].iter_mut()
    }

    pub fn unit(entry: E) -> Self {
        let mut moves = Self::empty();
        moves.push(entry);
        moves
    }

    pub fn push(&mut self, entry: E) {
        self.moves[self.top] = entry;
        self.top += 1;
    }

    pub fn consume_best(&mut self) -> Option<E> {
        if self.top == 0 {
            return None;
        }

        let mut best_idx = 0;
        let mut best_score = i16::MIN;

        for (idx, entry) in self.moves[0 .. self.top].iter().enumerate() {
            if entry.ordering_score() > best_score {
                best_score = entry.ordering_score();
                best_idx = idx;
            }
        }

        self.top -= 1;
        self.moves.swap(best_idx, self.top);

        Some(self.moves[self.top])
    }
}
