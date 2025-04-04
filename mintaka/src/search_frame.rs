use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::value::Score;

pub const KILLER_MOVE_SLOTS: usize = 2;

#[derive(Debug, Copy, Clone)]
pub struct SearchFrame {
    pub static_eval: Score,
    pub best_move: MaybePos,
    pub killer_moves: [MaybePos; KILLER_MOVE_SLOTS],
    pub extensions: bool,
    pub on_pv: bool,
}

impl Default for SearchFrame {

    fn default() -> Self {
        Self::EMPTY
    }

}

impl SearchFrame {

    const EMPTY: Self = SearchFrame {
        static_eval: 0,
        best_move: MaybePos::NONE,
        killer_moves: [MaybePos::NONE; KILLER_MOVE_SLOTS],
        extensions: false,
        on_pv: false,
    };

}
