use crate::movegen::movegen_window::MovegenWindow;
use rusty_renju::board::Board;
use rusty_renju::notation::pos::Pos;

#[derive(Debug, Copy, Clone)]
pub struct GameState {
    pub board: Board,
    pub history: [Pos; 256],
    pub history_top: usize,
    pub movegen_window: MovegenWindow,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            board: Default::default(),
            history: [Pos::INVALID; 256],
            history_top: 0,
            movegen_window: Default::default(),
        }
    }
}
