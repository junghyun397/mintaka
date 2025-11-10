use crate::game_state::RecoveryState;
use crate::search_endgame::EndgameAccumulator;
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

impl SearchFrame {
    pub const EMPTY: Self = Self {
        pos: MaybePos::NONE,
        static_eval: Score::ZERO,
        on_pv: false,
        recovery_state: RecoveryState::EMPTY,
        searching: MaybePos::NONE,
        cutoffs: 0,
    };
}
