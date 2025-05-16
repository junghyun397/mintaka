pub type Depth = u8;
pub type Score = i16;

pub trait Scores {
    const INF: Score = 32000;
    const WIN: Score = Self::INF - 1;

    fn win_in(ply: usize) -> Score {
        Self::WIN - ply as Score
    }

    fn lose_in(ply: usize) -> Score {
        -Self::WIN + ply as Score
    }

}

impl Scores for Score {}
