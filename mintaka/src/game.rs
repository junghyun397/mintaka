use crate::board::Board;
use crate::notation::color::Color;
use crate::notation::game_result::GameResult;
use crate::notation::history::History;
use crate::notation::pos::Pos;
use crate::notation::rule;

#[derive(Clone)]
pub struct Game {
    pub board: Board,
    pub history: History,
    pub result: Option<GameResult>,
    pub stones: usize,
}

impl Default for Game {

    fn default() -> Self {
        Self {
            board: Board::default(),
            history: History::default(),
            result: None,
            stones: 0,
        }
    }

}

impl Game {

    pub fn moves(&self) -> usize {
        self.history.len()
    }

    pub fn validate_move(&self, pos: Pos) -> bool {
        self.result.is_none()
            || !(self.board.slices.horizontal_slices[pos.row_usize()].stone_at(self.board.player_color, pos.col())
            || (self.board.player_color == Color::Black && self.board.formations.0[pos.idx_usize()].is_forbidden())
            || self.moves() == rule::BOARD_SIZE)
    }

    pub fn play(mut self, pos: Pos) -> Self {
        self.play_mut(pos);
        self
    }

    pub fn undo(mut self, pos: Pos) -> Self {
        self.undo_mut();
        self
    }

    pub fn pass(mut self) -> Self {
        self.pass_mut();
        self
    }

    pub fn play_mut(&mut self, pos: Pos) {
        self.board.set_mut(pos);
        self.stones += 1;

        self.history.play_mut(pos);
        self.result = self.board.winner
            .map(|color|
                GameResult::FiveInARow(color)
            )
            .or_else(||
                 (self.stones == rule::BOARD_SIZE).then(|| GameResult::Full)
            );
    }

    pub fn undo_mut(&mut self) {
        if let Some(pos) = self.history.undo_mut() {
            self.board.unset_mut(pos);
        }
        self.result = None;
    }

    pub fn pass_mut(&mut self) {
        self.board.pass_mut();
        self.history.pass_mut();
    }

    pub fn batch_set_mut(&mut self, blacks: Box<[Pos]>, whites: Box<[Pos]>) {
        let color = Color::player_color_by_moves(blacks.len(), whites.len());
        self.board.batch_set_mut(blacks, whites, color);
    }

}
