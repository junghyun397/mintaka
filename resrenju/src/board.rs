use crate::cache::hash_key::HashKey;
use crate::notation::color::Color;
use crate::formation::FormationPairs;
use crate::notation::game_result::GameResult;
use crate::notation::pos::Pos;
use crate::notation::rule::RuleKind;
use crate::slice::Slices;

// 1344-Bytes
#[derive(Copy, Clone)]
pub struct Board {
    pub moves: u8,
    pub slices: Slices,
    pub formation_pairs: FormationPairs,
    pub hash_key: HashKey,
}

impl Default for Board {

    fn default() -> Self {
        Self {
            moves: 0,
            slices: Default::default(),
            formation_pairs: Default::default(),
            hash_key: Default::default()
        }
    }

}

impl Board {

    pub fn player_color(&self) -> Color {
        match self.moves % 2 {
            1 => Color::White,
            _ => Color::Black
        }
    }

    pub fn next_color(&self) -> Color {
        match self.moves % 2 {
            1 => Color::Black,
            _ => Color::White
        }
    }

    pub fn set(&self, pos: Pos, rule_kind: RuleKind) -> Self {
        let color = self.player_color();
        let slices = self.slices.set(color, pos);

        Board {
            moves: self.moves + 1,
            slices,
            formation_pairs: todo!(),
            hash_key: self.hash_key.set(self.player_color(), pos),
        }
    }

    pub fn unset(&self, pos: Pos, rule_kind: RuleKind) -> Self {
        let color = self.player_color();
        let slices = self.slices.unset(color, pos);

        Board {
            moves: self.moves - 1,
            slices,
            formation_pairs: todo!(),
            hash_key: self.hash_key.set(self.player_color(), pos),
        }
    }

    pub fn set_mut(&mut self, pos: Pos, rule_kind: RuleKind) {
        self.hash_key = self.hash_key.set(self.player_color(), pos);
        self.moves += 1;
    }

    pub fn unset_mut(&mut self, pos: Pos, rule_kind: RuleKind) {
        self.hash_key = self.hash_key.set(self.player_color(), pos);
        self.moves -= 1;
    }

    pub fn batch_set_mut(&mut self, batch_pos: &Vec<Pos>, rule_kind: RuleKind) {
        self.moves += batch_pos.len() as u8;
    }

    pub fn find_result(&self) -> Option<GameResult> {
        todo!()
    }

}
