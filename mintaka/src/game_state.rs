use crate::movegen::move_scores::MoveScores;
use crate::movegen::movegen_window::MovegenWindow;
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::notation::pos::{MaybePos, Pos};

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
        self.movegen_window.imprint_window_mut(pos);
    }

    pub fn pass_mut(&mut self) {
        self.board.pass_mut();
        self.history.pass_mut();
    }

    pub fn unset_mut(&mut self, movegen_window: MovegenWindow) {
        let pos = self.history.pop_mut().unwrap().unwrap();
        self.board.unset_mut(pos);

        self.move_scores.remove_neighbor_score(pos);
        self.movegen_window = movegen_window;
    }

}

impl From<History> for GameState {
    fn from(value: History) -> Self {
        let mut game_state = GameState::default();

        for &maybe_pos in value.iter() {
            match maybe_pos {
                MaybePos::NONE => game_state.pass_mut(),
                pos => game_state.set_mut(pos.unwrap()),
            }
        }

        game_state
    }
}
