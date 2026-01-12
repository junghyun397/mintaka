use crate::value::MAX_PLY;
use rusty_renju::impl_debug_from_display;
use rusty_renju::notation::pos::MaybePos;
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::{Display, Formatter};
use typeshare::typeshare;

#[typeshare(serialized_as = "Vec<MaybePos>")]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PrincipalVariation {
    pub line: [MaybePos; MAX_PLY],
    pub top: usize,
}

impl PrincipalVariation {

    pub const EMPTY: Self = Self { line: [MaybePos::NONE; MAX_PLY], top: 0 };

    pub fn moves(&self) -> &[MaybePos] {
        &self.line[0 .. self.top]
    }

    pub fn clear(&mut self) {
        self.top = 0;
    }

    pub fn init(&mut self, head: MaybePos) {
        self.line[0] = head;
        self.top = 1;
    }

    pub fn load(&mut self, head: MaybePos, rest: Self) {
        self.line[0] = head;
        self.top = rest.top + 1;
        self.line[1 .. self.top].copy_from_slice(rest.moves());
    }

}

impl Display for PrincipalVariation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.moves())
    }
}

impl_debug_from_display!(PrincipalVariation);

#[cfg(feature = "serde")]
impl Serialize for PrincipalVariation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.collect_seq(self.moves())
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for PrincipalVariation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let vec = Vec::<MaybePos>::deserialize(deserializer)?;

        let mut line = [MaybePos::NONE; MAX_PLY];
        let top = vec.len();

        line[..top].copy_from_slice(&vec);

        Ok(Self { line, top })
    }
}
