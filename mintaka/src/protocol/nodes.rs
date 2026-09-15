use std::fmt::Display;
use std::ops::{Add, AddAssign, Div, Sub};

#[cfg_attr(feature = "typeshare", typeshare::typeshare)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Nodes {
    pub in_1k: u32,
}

impl Nodes {
    pub const ZERO: Self = Self { in_1k: 0 };

    pub const fn from_in_1k(in_1k: u32) -> Self {
        Self { in_1k }
    }
}

impl Add for Nodes {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            in_1k: self.in_1k + rhs.in_1k,
        }
    }
}

impl AddAssign for Nodes {
    fn add_assign(&mut self, rhs: Self) {
        self.in_1k += rhs.in_1k;
    }
}

impl Sub for Nodes {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            in_1k: self.in_1k - rhs.in_1k,
        }
    }
}

impl Div<u32> for Nodes {
    type Output = Self;
    fn div(self, rhs: u32) -> Self::Output {
        Self {
            in_1k: self.in_1k / rhs,
        }
    }
}

impl Display for Nodes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}K", self.in_1k)
    }
}
