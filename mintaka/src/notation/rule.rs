#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum RuleKind {
    FiveInARow = 0,
    SimplifiedRenju = 1,
    Renju = 2
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum ForbiddenKind {
    DoubleThree,
    DoubleFour,
    Overline,
}
