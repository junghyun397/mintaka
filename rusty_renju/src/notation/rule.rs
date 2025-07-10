use crate::impl_debug_from_display;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::marker::ConstParamTy;

//noinspection RsUnresolvedPath
#[derive(ConstParamTy, Eq, PartialEq, Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub enum RuleKind {
    Gomoku = 0,
    SimplifiedRenju = 1,
    #[default] Renju = 2
}

impl RuleKind {

    pub const fn relaxed(&self) -> Self {
        match self {
            Self::Renju => Self::SimplifiedRenju,
            Self::SimplifiedRenju => Self::Gomoku,
            _ => Self::Gomoku
        }
    }

    pub const fn stricter(&self) -> Self {
        match self {
            Self::SimplifiedRenju => Self::Renju,
            Self::Gomoku => Self::SimplifiedRenju,
            _ => Self::Renju
        }
    }

}

#[derive(PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
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

impl Display for ForbiddenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", char::from(*self))
    }
}

impl_debug_from_display!(ForbiddenKind);
