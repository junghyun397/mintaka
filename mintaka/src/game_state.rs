use crate::movegen::move_scores::MoveScores;
use crate::movegen::movegen_window::MovegenWindow;
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::notation::pos::Pos;

#[derive(Default, Debug, Copy, Clone)]
pub struct GameState {
    pub board: Board,
    pub history: History,
    pub movegen_window: MovegenWindow,
    pub move_scores: MoveScores,
}

impl GameState {

    pub fn set(&mut self, pos: Pos) {
        self.board.set_mut(pos);
        self.history.set_mut(pos);

        self.move_scores.add_neighborhood_score(pos);
        self.movegen_window.expand_window_mut(pos);
    }

    pub fn pass_unchecked(&mut self) {
        self.board.pass_mut();
    }

    pub fn unset(&mut self, movegen_window: MovegenWindow) {
        let pos = self.history.pop().unwrap().unwrap();
        self.board.unset_mut(pos);

        self.move_scores.remove_neighborhood_score(pos);
        self.movegen_window = movegen_window;
    }

}
