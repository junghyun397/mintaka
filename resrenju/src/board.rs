use crate::cache::hash_key::HashKey;
use crate::notation::color::Color;
use crate::notation::game_result::GameResult;
use crate::notation::pos::Pos;
use crate::notation::rule::RuleKind;
use crate::slice::Slices;

const SLICE_AMOUNT: usize = 0;

#[derive(Copy, Clone)]
pub struct Board {
    pub moves: u8,
    pub slices: Slices,
    pub hash_key: HashKey
}

impl Board {

    pub fn empty() -> Self {
        Board {
            moves: 0,
            slices: Slices::empty(),
            hash_key: HashKey::empty()
        }
    }

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
        todo!()
    }

    pub fn unset(&self, pos: Pos, rule_kind: RuleKind) -> Self {
        todo!()
    }

    pub fn set_mut(&mut self, pos: Pos, rule_kind: RuleKind) {
        todo!()
    }

    pub fn unset_mut(&mut self, pos: Pos, rule_kind: RuleKind) {
        todo!()
    }

    pub fn batch_set_mut(&mut self, batch_pos: &Vec<Pos>, rule_kind: RuleKind) {
        todo!()
    }

    pub fn find_result(&self) -> Option<GameResult> {
        todo!()
    }

}
