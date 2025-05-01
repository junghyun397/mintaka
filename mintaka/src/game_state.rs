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

    pub fn set_mut(&mut self, pos: Pos) {
        self.board.set_mut(pos);
        self.history.set_mut(pos);

        self.move_scores.add_neighbor_score(pos);
        self.movegen_window.expand_window_mut(pos);
    }

    pub fn pass_mut(&mut self) {
        self.board.pass_mut();
    }

    pub fn unset_mut(&mut self, movegen_window: MovegenWindow) {
        let pos = self.history.pop_mut().unwrap().unwrap();
        self.board.unset_mut(pos);

        self.move_scores.remove_neighbor_score(pos);
        self.movegen_window = movegen_window;
    }

}
