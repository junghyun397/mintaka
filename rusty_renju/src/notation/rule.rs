use std::marker::ConstParamTy;

//noinspection RsUnresolvedPath
#[derive(ConstParamTy, Eq, PartialEq, Copy, Clone, Debug, Default)]
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

    pub const fn strict(&self) -> Self {
        match self {
            Self::SimplifiedRenju => Self::Renju,
            Self::Gomoku => Self::SimplifiedRenju,
            _ => Self::Renju
        }
    }

}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum ForbiddenKind {
    DoubleThree = 1,
    DoubleFour = 2,
    Overline = 3,
}
