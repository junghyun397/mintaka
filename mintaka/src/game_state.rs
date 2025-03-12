use crate::movegen::movegen_window::MovegenWindow;
use rusty_renju::board::Board;

#[derive(Debug, Copy, Clone)]
pub struct GameState {
    pub board: Board,
    pub movegen_window: MovegenWindow,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            board: Default::default(),
            movegen_window: Default::default(),
        }
    }
}
