use crate::notation::color::Color;
use crate::notation::pos::Pos;

pub enum OpeningStage {
    Move {
        move_window_width: usize,
    },
    Swap {
        usize: usize,
        color: Color
    },
    Declare {
        min_candidates: usize,
        max_candidates: usize
    },
    Offer {
        min_candidates: usize,
        max_candidates: usize
    },
    Select {
        candidates: Vec<Pos>
    },
    Branch
}
