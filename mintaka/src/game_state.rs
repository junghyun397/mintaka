use crate::movegen::movegen_window::MovegenWindow;
use rusty_renju::board::Board;
use rusty_renju::notation::pos::Pos;

#[derive(Debug, Copy, Clone)]
pub struct GameState {
    pub board: Board,
    pub recent_move: Pos,
    pub movegen_window: MovegenWindow,
}
