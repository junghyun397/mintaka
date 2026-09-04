use rusty_renju::notation::pos;
use std::fmt::Display;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

pub const MAX_PLY: usize = 128;
pub const MAX_PLY_SLOTS: usize = MAX_PLY + 1;

#[cfg_attr(feature = "typeshare", typeshare::typeshare)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Depth(i32);

impl Depth {
    pub const ZERO: Depth = Depth(0);
    pub const PLY_LIMIT: Depth = Depth(MAX_PLY as i32);
    pub const BOARD_LIMIT: Depth = Depth(pos::BOARD_SIZE as i32);

    pub const fn value(self) -> i32 {
        self.0
    }

    pub const fn from_i32(value: i32) -> Self {
        debug_assert!(Self::PLY_LIMIT.0 > value && value >= 0);
        Self(value)
    }

    pub fn clamp(self) -> Self {
        Self(self.0.clamp(0, Self::PLY_LIMIT.0))
    }

    pub fn clamp_value(self, value: Depth) -> Self {
        Self(self.0.clamp(0, value.0))
    }
}

impl From<i32> for Depth {
    fn from(value: i32) -> Self {
        Self::from_i32(value)
    }
}

impl Add<Depth> for Depth {
    type Output = Self;
    fn add(self, rhs: Depth) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Add<i32> for Depth {
    type Output = Self;
    fn add(self, rhs: i32) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<Depth> for Depth {
    fn add_assign(&mut self, rhs: Depth) {
        self.0 += rhs.0
    }
}

impl Sub<Depth> for Depth {
    type Output = Self;
    fn sub(self, rhs: Depth) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Sub<i32> for Depth {
    type Output = Self;
    fn sub(self, rhs: i32) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl SubAssign<Depth> for Depth {
    fn sub_assign(&mut self, rhs: Depth) {
        self.0 -= rhs.0
    }
}

impl Mul<i32> for Depth {
    type Output = i32;
    fn mul(self, rhs: i32) -> Self::Output {
        self.0 * rhs
    }
}

impl Div<i32> for Depth {
    type Output = i32;
    fn div(self, rhs: i32) -> Self::Output {
        self.0 / rhs
    }
}

impl Display for Depth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
