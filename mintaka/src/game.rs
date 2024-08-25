use crate::board::Board;
use crate::notation::game_result::GameResult;
use crate::notation::history::History;
use crate::notation::pos::Pos;
use crate::notation::rule::RuleKind;

pub struct Game {
    pub board: Board,
    pub history: History,
    pub result: Option<GameResult>,
}

impl Default for Game {

    fn default() -> Self {
        Self {
            board: Default::default(),
            history: Default::default(),
            result: None
        }
    }

}

impl Game {

    pub fn play(&self, pos: Pos, rule_kind: RuleKind) -> Self {
        Self {
            board: self.board.set(pos, rule_kind),
            history: self.history.play(pos),
            result: None
        }
    }

    pub fn undo(&self, pos: Pos, rule_kind: RuleKind) -> Self {
        Self {
            board: self.board.unset(pos, rule_kind),
            history: self.history.undo(),
            result: None
        }
    }

    pub fn play_mut(&mut self, pos: Pos, rule_kind: RuleKind) {
        self.history.play_mut(pos);
        todo!()
    }

    pub fn undo_mut(&mut self, pos: Pos, rule_kind: RuleKind) {
        self.history.undo_mut();
        todo!()
    }

}
