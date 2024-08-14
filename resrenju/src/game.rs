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

impl Game {

    pub fn empty() -> Self {
        Game {
            board: Board::empty(),
            history: History::empty(),
            result: None
        }
    }

    pub fn play(&self, pos: Pos, rule_kind: RuleKind) -> Game {
        todo!()
    }

    pub fn undo(&self, pos: Pos, rule_kind: RuleKind) -> Game {
        todo!()
    }

    pub fn play_mut(&mut self, pos: Pos, rule_kind: RuleKind) {
        todo!()
    }

    pub fn undo_mut(&mut self, pos: Pos, rule_kind: RuleKind) {
        todo!()
    }

}
