use std::fmt::Display;
#[cfg(feature = "typeshare")]
use typeshare::typeshare;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(std::marker::ConstParamTy, Default, PartialEq, Eq, Copy, Clone, Debug)]
pub enum RuleKind {
    #[default] Renju = 0,
    Gomoku = 1,
    Freestyle = 2,
}

impl Display for RuleKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Renju => write!(f, "Renju"),
            Self::Gomoku => write!(f, "Gomoku"),
            Self::Freestyle => write!(f, "Freestyle"),
        }
    }
}

#[cfg_attr(feature = "typeshare", typeshare(serialized_as = "String"))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[repr(u8)]
pub enum ForbiddenKind {
    DoubleThree = 1,
    DoubleFour = 2,
    Overline = 3,
}

impl From<ForbiddenKind> for char {
    fn from(value: ForbiddenKind) -> Self {
        match value {
            ForbiddenKind::DoubleThree => crate::board_io::SYMBOL_FORBID_DOUBLE_THREE,
            ForbiddenKind::DoubleFour => crate::board_io::SYMBOL_FORBID_DOUBLE_FOUR,
            ForbiddenKind::Overline => crate::board_io::SYMBOL_FORBID_OVERLINE
        }
    }
}
