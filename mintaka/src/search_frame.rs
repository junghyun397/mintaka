use crate::game_state::RecoveryState;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::score::Score;

pub const KILLER_MOVE_SLOTS: usize = 2;

#[derive(Debug, Copy, Clone)]
pub struct SearchFrame {
    pub hash_key: HashKey,
    pub static_eval: Score,
    pub on_pv: bool,
    pub recovery_state: RecoveryState,
    pub searching: MaybePos,
    pub cutoffs: usize,
}
