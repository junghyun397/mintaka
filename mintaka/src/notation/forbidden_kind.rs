#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ForbiddenKind {
    DoubleThree = 0b0100,
    DoubleFour = 0b0010,
    Overline = 0b0001,
}
