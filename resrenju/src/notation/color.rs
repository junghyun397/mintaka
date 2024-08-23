#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Color {
    Black,
    White
}

impl Color {

    pub fn reversed(&self) -> Color {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black
        }
    }

}
