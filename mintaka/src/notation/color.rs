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

    pub fn player_color_by_moves<T: Ord>(black_moves: T, white_moves: T) -> Self {
        if black_moves > white_moves {
            Color::Black
        } else {
            Color::White
        }
    }

}
