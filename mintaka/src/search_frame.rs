use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::value::Score;

pub const KILLER_MOVE_SLOTS: usize = 2;

#[derive(Debug, Copy, Clone)]
pub struct SearchFrame {
    pub pos: MaybePos,
    pub static_eval: Score,
    pub on_pv: bool,
    pub cutoffs: usize,
}
