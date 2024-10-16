use crate::bitfield::BitfieldOps;
use crate::board::Board;
use crate::history::History;
use crate::notation::color::Color;
use crate::notation::pos;
use crate::notation::pos::Pos;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum GameResult {
    FiveInARow(Color),
    Resign(Color),
    Full,
}

#[derive(Clone, Default)]
pub struct Game {
    pub board: Board,
    pub history: History,
    pub result: Option<GameResult>,
}

impl Game {

    pub fn moves(&self) -> usize {
        self.history.len()
    }

    pub fn validate_move(&self, pos: Pos) -> bool {
        !(self.result.is_some()
            || self.board.hot_field.is_hot(pos)
            || (self.board.player_color == Color::Black && self.board.patterns.field[pos.idx_usize()].is_forbidden())
            || self.moves() == pos::BOARD_SIZE
        )
    }

    pub fn play(mut self, pos: Pos) -> Self {
        self.play_mut(pos);
        self
    }

    pub fn undo(mut self) -> Self {
        self.undo_mut();
        self
    }

    pub fn pass(mut self) -> Self {
        self.pass_mut();
        self
    }

    pub fn resign(mut self, resigned_player: Color) -> Self {
        self.resign_mut(resigned_player);
        self
    }

    pub fn play_mut(&mut self, pos: Pos) {
        self.board.set_mut(pos);

        self.history.play_mut(pos);
        self.result = self.board.patterns.five_in_a_row
            .map(|(_, _, color)| GameResult::FiveInARow(color))
            .or_else(||
                 (self.board.stones == pos::U8_BOARD_SIZE).then_some(GameResult::Full)
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

    pub fn resign_mut(&mut self, resigned_player: Color) {
        self.result = Some(GameResult::Resign(resigned_player.reversed()));
    }

    pub fn batch_set_each_mut(&mut self, blacks: Box<[Pos]>, whites: Box<[Pos]>) {
        let color = Color::player_color_from_each_moves(blacks.len(), whites.len());
        self.board.batch_set_each_color_mut(blacks, whites, color);
    }

}
