use serde::{Deserialize, Serialize};

#[derive(std::marker::ConstParamTy, Default, PartialEq, Eq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum RuleKind {
    Gomoku = 0,
    #[default] Renju = 1
}

impl RuleKind {

    pub const fn relaxed(&self) -> Self {
        match self {
            Self::Renju => Self::Gomoku,
            _ => Self::Gomoku
        }
    }

    pub const fn stricter(&self) -> Self {
        match self {
            Self::Gomoku => Self::Renju,
            _ => Self::Renju
        }
    }

}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Serialize, Deserialize)]
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
