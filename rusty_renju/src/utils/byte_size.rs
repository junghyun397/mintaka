use crate::impl_debug_from_display;
use std::fmt::Display;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ByteSize(usize);

impl ByteSize {

    pub fn from_bytes(size_in_bytes: usize) -> Self {
        Self(size_in_bytes)
    }

    pub fn from_kib(size_in_kib: usize) -> Self {
        Self(size_in_kib * 1024)
    }

    pub fn from_mib(size_in_mib: usize) -> Self {
        Self(size_in_mib * 1024 * 1024)
    }

    pub fn bytes(&self) -> usize {
        self.0
    }

    pub fn kib(&self) -> usize {
        self.0 / 1024
    }

    pub fn mib(&self) -> usize {
        self.0 / (1024 * 1024)
    }

}

impl From<usize> for ByteSize {
    fn from(size_in_bytes: usize) -> Self {
        Self(size_in_bytes)
    }
}

impl Display for ByteSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = self.0;
        match bytes {
            b if b < 1024 => write!(f, "{}", b),
            b if b < 1024 * 1024 => write!(f, "{:.2} KiB", b as f64 / 1024.0),
            b => write!(f, "{:.2} MiB", b as f64 / (1024.0 * 1024.0)),
        }
    }
}

impl_debug_from_display!(ByteSize);
