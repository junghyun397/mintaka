use crate::movegen::movegen_window::MovegenWindow;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::value::Score;

pub const KILLER_MOVE_SLOTS: usize = 2;

#[derive(Debug, Copy, Clone)]
pub struct SearchFrame {
    pub hash_key: HashKey,
    pub static_eval: Score,
    pub on_pv: bool,
    pub movegen_window: MovegenWindow,
    pub last_pos: MaybePos,
    pub cutoffs: usize,
}
