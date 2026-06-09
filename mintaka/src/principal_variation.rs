use crate::value::MAX_PLY;
use rusty_renju::impl_debug_from_display;
use rusty_renju::notation::pos::MaybePos;
use std::fmt::{Display, Formatter};
#[cfg(feature = "typeshare")]
use typeshare::typeshare;

#[cfg_attr(feature = "typeshare", typeshare(serialized_as = "Vec<MaybePos>"))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "Vec<MaybePos>"))]
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

impl TryFrom<Vec<MaybePos>> for PrincipalVariation {
    type Error = &'static str;

    fn try_from(vec: Vec<MaybePos>) -> Result<Self, Self::Error> {
        let top = vec.len();
        
        if top > MAX_PLY {
            return Err("moves longer than max ply");
        }

        let mut line = [MaybePos::NONE; MAX_PLY];
        line[..top].copy_from_slice(&vec);

        Ok(Self { line, top })
    }
}

impl From<&PrincipalVariation> for Vec<MaybePos> {
    fn from(pv: &PrincipalVariation) -> Self {
        pv.moves().to_vec()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for PrincipalVariation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        Vec::from(self).serialize(serializer)
    }
}
