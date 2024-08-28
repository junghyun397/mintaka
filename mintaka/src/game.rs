use crate::board::Board;
use crate::notation::color::Color;
use crate::notation::game_result::GameResult;
use crate::notation::history::History;
use crate::notation::pos::Pos;

#[derive(Clone)]
pub struct Game {
    pub board: Board,
    pub history: History,
    pub result: Option<GameResult>,
}

impl Default for Game {

    fn default() -> Self {
        Self {
            board: Board::default(),
            history: History::default(),
            result: None
        }
    }

}

impl Game {

    pub fn moves(&self) -> usize {
        self.history.0.len()
    }

    pub fn play(&self, pos: Pos) -> Self {
        let mut game = self.clone();
        game.play_mut(pos);
        game
    }

    pub fn undo(&self, pos: Pos) -> Self {
        let mut game = self.clone();
        game.undo_mut(pos);
        game
    }

    pub fn pass(&self) -> Self {
        let mut game = self.clone();
        game.pass_mut();
        game
    }

    pub fn play_mut(&mut self, pos: Pos) {
        self.board.set_mut(pos);
        self.result = None;
        self.history.play_mut(pos);
    }

    pub fn undo_mut(&mut self, pos: Pos) {
        self.board.unset_mut(pos);
        self.result = None;
        self.history.undo_mut();
    }

    pub fn pass_mut(&mut self) {
        self.board.pass_mut();
        self.history.pass_mut();
    }

    pub fn batch_set_mut(&mut self, blacks: Vec<Pos>, whites: Vec<Pos>) {
        let color = Color::player_color_by_moves(blacks.len(), whites.len());
        self.board.batch_set_mut(blacks, whites, color);
    }

}
