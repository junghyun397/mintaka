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

    fn action_mut(&mut self, action: MaybePos) {
        self.entries[self.top] = action;
        self.top += 1;
    }

    pub fn set_mut(&mut self, pos: Pos) {
        self.action_mut(pos.into())
    }

    pub fn pass_mut(&mut self) {
        self.action_mut(MaybePos::NONE)
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

    pub fn recent_player_move_unchecked(&self) -> Pos {
        self.entries[self.top - 2].unwrap()
    }

    pub fn recent_opponent_move_unchecked(&self) -> Pos {
        self.entries[self.top - 1].unwrap()
    }

    pub fn recent_move_pair_unchecked(&self) -> [Pos; 2] {
        [self.recent_player_move_unchecked(), self.recent_opponent_move_unchecked()]
    }

    pub fn multi_distance(&self, pos: Pos) -> u8 {
        match self.top {
            1 => self.entries[0].unwrap().distance(pos),
            2 => {
                let distance1 = self.entries[self.top - 2].unwrap().distance(pos);
                let distance2 = self.entries[self.top - 1].unwrap().distance(pos);
                (distance1 + distance2) / 2
            },
            3 => {
                let distance1 = self.entries[self.top - 3].unwrap().distance(pos);
                let distance2 = self.entries[self.top - 2].unwrap().distance(pos);
                let distance3 = self.entries[self.top - 1].unwrap().distance(pos);
                (distance1 + distance2 + distance3) / 3
            },
            _ => {
                let distance1 = self.entries[self.top - 4].unwrap().distance(pos);
                let distance2 = self.entries[self.top - 3].unwrap().distance(pos);
                let distance3 = self.entries[self.top - 2].unwrap().distance(pos);
                let distance4 = self.entries[self.top - 1].unwrap().distance(pos);
                (distance1 + distance2 + distance3 + distance4) / 4
            }
        }
    }

}
