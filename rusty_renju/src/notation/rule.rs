use std::marker::ConstParamTy;

//noinspection RsUnresolvedPath
#[derive(ConstParamTy, Eq, PartialEq, Copy, Clone, Debug, Default)]
pub enum RuleKind {
    Gomoku = 0,
    SimplifiedRenju = 1,
    #[default] Renju = 2
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum ForbiddenKind {
    DoubleThree = 1,
    DoubleFour = 2,
    Overline = 3,
}
