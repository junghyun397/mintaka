use crate::board::Board;
use crate::notation::color::Color;
use crate::notation::game_result::GameResult;
use crate::notation::history::History;
use crate::notation::pos::Pos;
use crate::notation::rule::RuleKind;

#[derive(Clone)]
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

    pub fn moves(&self) -> usize {
        self.history.0.len()
    }

    pub fn play(&self, pos: Pos, rule_kind: RuleKind) -> Self {
        let mut game = self.clone();
        game.play_mut(pos, rule_kind);
        game
    }

    pub fn undo(&self, pos: Pos, rule_kind: RuleKind) -> Self {
        let mut game = self.clone();
        game.undo_mut(pos, rule_kind);
        game
    }

    pub fn pass(&self) -> Self {
        let mut game = self.clone();
        game.pass_mut();
        game
    }

    pub fn play_mut(&mut self, pos: Pos, rule_kind: RuleKind) {
        self.board.set_mut(pos, rule_kind);
        self.result = None;
        self.history.play_mut(pos);
    }

    pub fn undo_mut(&mut self, pos: Pos, rule_kind: RuleKind) {
        self.board.unset_mut(pos, rule_kind);
        self.result = None;
        self.history.undo_mut();
    }

    pub fn pass_mut(&mut self) {
        self.board.pass_mut();
        self.history.pass_mut();
    }

    pub fn batch_set_mut(&mut self, blacks: Vec<Pos>, whites: Vec<Pos>, next_player: Color, rule_kind: RuleKind) {
        self.board.batch_set_mut(blacks, whites, next_player, rule_kind);
    }

}
