pub type Score = i32;

pub trait Scores {
    const INF: Score = i16::MAX as i32;
    const WIN: Score = i16::MAX as i32 - 1;
    const DETERMINISTIC: Score = Score::WIN - 400;
    const DRAW: Score = 0;

    fn win_in(ply: usize) -> Score {
        Self::WIN - ply as Score
    }

    fn lose_in(ply: usize) -> Score {
        ply as Score - Self::WIN
    }

    fn is_deterministic(score: Score) -> bool {
        !(-Score::DETERMINISTIC ..= Score::DETERMINISTIC).contains(&score)
    }

}

impl Scores for Score {}
