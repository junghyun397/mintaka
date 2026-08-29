use std::fmt::Display;
use std::ops::{Add, Div, Mul, Neg, Sub};
use crate::notation::pos;

#[cfg_attr(feature = "typeshare", typeshare::typeshare)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Score(i32);

impl Score {
    pub const NAN: Score = Score(-i16::MAX as i32);
    pub const INF: Score = Score(i16::MAX as i32 - 1);
    pub const NEG_INF: Score = Score(-i16::MAX as i32 - 1);
    pub const WIN: Score = Score(32000);
    pub const MATE_LIMIT: Score = Score(Score::WIN.0 - pos::BOARD_SIZE as i32);
    pub const ABORT: Score = Score(0);
    pub const DRAW: Score = Score(0);

    pub fn from_i32_clamp(value: i32) -> Score {
        assert_ne!(value, Self::NAN.0);
        Score(value.clamp(Self::NEG_INF.0, Self::INF.0))
    }

    pub const fn from_i32_unchecked(value: i32) -> Score {
        Score(value)
    }

    pub fn is_nan(&self) -> bool {
        self.0 == Self::NAN.0
    }

    pub fn is_mate(&self) -> bool {
        self.0.abs() >= Self::MATE_LIMIT.0
    }

    pub fn is_win(&self) -> bool {
        self.0 >= Self::MATE_LIMIT.0
    }

    pub fn is_lose(&self) -> bool {
        self.0 <= -Self::MATE_LIMIT.0
    }

    pub fn unwrap(self) -> i32 {
        assert_ne!(self.0, Self::NAN.0);
        self.0
    }

    pub const fn unwrap_unchecked(self) -> i32 {
        self.0
    }

    pub fn fallback(self, fallback: Score) -> Score {
        if self.0 == Score::NAN.0 {
            fallback
        } else {
            self
        }
    }

    pub fn win_in(ply: usize) -> Score {
        Score(Self::WIN.0 - ply as i32)
    }

    pub fn lose_in(ply: usize) -> Score {
        Score(ply as i32 - Self::WIN.0)
    }
}

impl Neg for Score {
    type Output = Score;

    fn neg(self) -> Self::Output {
        Score(-self.0)
    }
}

impl Add<Score> for Score {
    type Output = Score;

    fn add(self, rhs: Self) -> Self::Output {
        Score(self.0 + rhs.0)
    }
}

impl Add<i32> for Score {
    type Output = Score;

    fn add(self, rhs: i32) -> Self::Output {
        Score(self.0 + rhs)
    }
}

impl Sub<Score> for Score {
    type Output = Score;

    fn sub(self, rhs: Self) -> Self::Output {
        Score((self.0 - rhs.0).max(Score::NEG_INF.0))
    }
}

impl Sub<i32> for Score {
    type Output = Score;

    fn sub(self, rhs: i32) -> Self::Output {
        Score((self.0 - rhs).max(Score::NEG_INF.0))
    }
}

impl Div<i32> for Score {
    type Output = Score;

    fn div(self, rhs: i32) -> Self::Output {
        Score((self.0 / rhs).min(Score::INF.0))
    }
}

impl Mul<i32> for Score {
    type Output = Score;

    fn mul(self, rhs: i32) -> Self::Output {
        Score((self.0 * rhs).min(Score::INF.0))
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
