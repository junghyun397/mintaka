use crate::movegen::movegen_window::MovegenWindow;
use rusty_renju::board::Board;

#[derive(Default, Debug, Copy, Clone)]
pub struct GameState {
    pub board: Board,
    pub movegen_window: MovegenWindow,
}
