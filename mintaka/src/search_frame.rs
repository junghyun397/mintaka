use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::value::Score;

pub const KILLER_MOVE_SLOTS: usize = 2;

#[derive(Debug, Copy, Clone)]
pub struct SearchFrame {
    pub pos: MaybePos,
    pub best_move: MaybePos,
    pub static_eval: Score,
    pub on_pv: bool,
}

impl Default for SearchFrame {

    fn default() -> Self {
        Self::EMPTY
    }

}

impl SearchFrame {

    pub const EMPTY: Self = SearchFrame {
        pos: MaybePos::NONE,
        best_move: MaybePos::NONE,
        static_eval: 0,
        on_pv: false,
    };

}
