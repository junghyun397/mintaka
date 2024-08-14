pub enum Color {
    Black = 0,
    White = 1
}

impl Color {

    pub fn reverse(self) -> Color {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black
        }
    }

}
