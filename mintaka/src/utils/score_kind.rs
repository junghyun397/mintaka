use rusty_renju::notation::score::Score;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum ScoreKind {
    UpperBound = 1,
    LowerBound = 2,
    Exact = 3,
}

impl ScoreKind {
    pub fn from_alpha_beta(score: Score, alpha: Score, beta: Score) -> Self {
        if score >= beta {
            Self::LowerBound
        } else if score > alpha {
            Self::Exact
        } else {
            Self::UpperBound
        }
    }
}

impl From<ScoreKind> for u8 {
    fn from(score_kind: ScoreKind) -> Self {
        score_kind as u8
    }
}

impl From<ScoreKind> for i32 {
    fn from(score_kind: ScoreKind) -> Self {
        score_kind as i32
    }
}
