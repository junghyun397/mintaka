pub type Depth = u8;
pub type Score = i16;

pub trait Scores {
    const INF: Score = 32000;
    const WIN: Score = Self::INF - 1;
}

impl Scores for Score {}
