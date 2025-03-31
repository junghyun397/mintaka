use crate::movegen::movegen_window::MovegenWindow;
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::notation::pos::Pos;

#[derive(Default, Debug, Copy, Clone)]
pub struct GameState {
    pub board: Board,
    pub movegen_window: MovegenWindow,
    pub history: History,
}

impl GameState {

    pub fn set(&mut self, pos: Pos) {
        self.board.set(pos);
        self.movegen_window.expand_window_mut(pos);
        self.history.push(pos.into());
    }

    pub fn unset(&mut self) {
        let pos = self.history.pop().unwrap();
        self.board.unset(pos.unwrap());
    }

}
