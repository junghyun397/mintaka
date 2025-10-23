pub type Score = i32;

pub trait Scores {
    const NAN: Score = -i16::MAX as Score;
    const INF: Score = i16::MAX as Score - 1;
    const WIN: Score = 32000;
    const DETERMINISTIC: Score = Score::WIN - 300;
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

    fn is_winning(score: Score) -> bool {
        score >= Self::DETERMINISTIC
    }

    fn clamp(score: Score) -> Score {
        score.clamp(-Score::INF, Score::INF)
    }

}

impl Scores for Score {}
