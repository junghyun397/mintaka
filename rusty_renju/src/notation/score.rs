use crate::notation::pos;

#[cfg_attr(feature = "typeshare", typeshare::typeshare)]
pub type Score = i32;

pub trait Scores {
    const NAN: Score = -i16::MAX as Score;
    const INF: Score = i16::MAX as Score - 1;
    const WIN: Score = 32000;
    const MATE_LIMIT: Score = Score::WIN - pos::BOARD_SIZE as Score;
    const ABORT: Score = 0;
    const DRAW: Score = 0;

    fn win_in(ply: usize) -> Score {
        Self::WIN - ply as Score
    }

    fn lose_in(ply: usize) -> Score {
        ply as Score - Self::WIN
    }

    fn is_mate(score: Score) -> bool {
        score.abs() >= Self::MATE_LIMIT
    }

    fn is_winning(score: Score) -> bool {
        score >= Self::MATE_LIMIT
    }

    fn is_losing(score: Score) -> bool {
        score <= -Self::MATE_LIMIT
    }

    fn clamp(score: Score) -> Score {
        score.clamp(-Score::INF, Score::INF)
    }
}

impl Scores for Score {}
