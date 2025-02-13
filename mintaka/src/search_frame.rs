use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::value::Eval;

#[derive(Debug, Copy, Clone)]
pub struct SearchFrame {
    pub static_eval: Eval,
    pub best_move: Option<Pos>,
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
        best_move: None,
        extensions: false,
        on_pv: false,
    };

}
