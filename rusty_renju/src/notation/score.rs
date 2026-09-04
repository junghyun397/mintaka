use std::fmt::Display;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

#[cfg_attr(feature = "typeshare", typeshare::typeshare)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Score(i32);

impl Score {
    pub const DRAW: Score = Score(0);
    pub const MATE: Score = Score(32000);
    pub const NEG_MATE: Score = Score(-Self::MATE.0);
    pub const INF: Score = Score(32001);
    pub const NEG_INF: Score = Score(-Self::INF.0);
    pub const MATE_MIN: Score = Score(32000 - 256);

    pub const fn from_i32(value: i32) -> Self {
        debug_assert!(Self::MATE.0 >= value && value >= Self::NEG_INF.0);
        Self(value)
    }

    pub const fn value(self) -> i32 {
        self.0
    }

    pub fn clamp(self) -> Self {
        Self(self.0.clamp(-Self::MATE.0, Self::MATE.0))
    }

    pub fn clamp_non_mate(self) -> Self {
        Self(self.0.clamp(-Self::MATE_MIN.0 + 1, Self::MATE_MIN.0 - 1))
    }

    pub const fn is_mate(&self) -> bool {
        self.0.abs() >= Self::MATE_MIN.0
    }

    pub const fn is_win(&self) -> bool {
        self.0 >= Self::MATE_MIN.0
    }

    pub const fn is_lose(&self) -> bool {
        self.0 <= -Self::MATE_MIN.0
    }

    pub const fn win_in(ply: usize) -> Self {
        Self(Self::MATE.0 - ply as i32)
    }

    pub const fn lose_in(ply: usize) -> Self {
        Self(ply as i32 - Self::MATE.0)
    }
}

impl Neg for Score {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Add<Score> for Score {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Add<i32> for Score {
    type Output = Self;
    fn add(self, rhs: i32) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<Score> for Score {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl Sub<Score> for Score {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Sub<i32> for Score {
    type Output = Self;
    fn sub(self, rhs: i32) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl SubAssign<Score> for Score {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0
    }
}

impl Div<i32> for Score {
    type Output = Self;
    fn div(self, rhs: i32) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl Mul<i32> for Score {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i32> for Score {
    fn from(value: i32) -> Self {
        Self::from_i32(value)
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MaybeScore(pub Score);

impl MaybeScore {
    pub const INVALID_SCORE: Score = Score(i16::MIN as i32);

    pub const NONE: Self = Self(Self::INVALID_SCORE);

    pub fn is_none(&self) -> bool {
        self.0 == Self::INVALID_SCORE
    }

    pub fn is_some(&self) -> bool {
        self.0 != Self::INVALID_SCORE
    }

    pub fn or(self, other: Self) -> Self {
        if self.is_some() { self } else { other }
    }

    pub fn unwrap(self) -> Score {
        assert!(self.is_some());
        self.0
    }

    pub fn unwrap_or(self, default: Score) -> Score {
        if self.is_some() { self.0 } else { default }
    }

    pub fn unwrap_unchecked(self) -> i32 {
        self.0.0
    }
}

impl From<Score> for MaybeScore {
    fn from(score: Score) -> Self {
        Self(score)
    }
}

impl From<i32> for MaybeScore {
    fn from(score: i32) -> Self {
        if score == Self::INVALID_SCORE.0 {
            Self::NONE
        } else {
            Self(Score::from_i32(score))
        }
    }
}
