use crate::game_state::RecoveryState;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::score::Score;

#[derive(Debug, Copy, Clone)]
pub struct SearchFrame {
    pub pos: MaybePos,
    pub static_eval: Score,
    pub on_pv: bool,
    pub recovery_state: RecoveryState,
    pub searching: MaybePos,
    pub cutoffs: usize,
}
