use crate::notation::color::Color;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum GameResult {
    FiveInARow(Color),
    Resign(Color),
    Full,
}
