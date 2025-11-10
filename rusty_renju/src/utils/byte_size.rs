use crate::impl_debug_from_display;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Clone, Copy, PartialOrd, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ByteSize(usize);

impl ByteSize {

    pub const ZERO: Self = Self(0);

    pub const fn from_bytes(size_in_bytes: usize) -> Self {
        Self(size_in_bytes)
    }

    pub const fn from_kib(size_in_kib: usize) -> Self {
        Self(size_in_kib * 1024)
    }

    pub const fn from_mib(size_in_mib: usize) -> Self {
        Self(size_in_mib * 1024 * 1024)
    }

    pub const fn bytes(&self) -> usize {
        self.0
    }

    pub const fn kib(&self) -> usize {
        self.0 / 1024
    }

    pub const fn mib(&self) -> usize {
        self.0 / (1024 * 1024)
    }

}

impl Add for ByteSize {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for ByteSize {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for ByteSize {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for ByteSize {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Display for ByteSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = self.0;
        match bytes {
            b if b < 1024 => write!(f, "{b} B"),
            b if b < 1024 * 1024 => write!(f, "{:.2} KiB", b as f64 / 1024.0),
            b => write!(f, "{:.2} MiB", b as f64 / (1024.0 * 1024.0)),
        }
    }
}

impl_debug_from_display!(ByteSize);
