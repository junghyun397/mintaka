use std::marker::ConstParamTy;

//noinspection RsUnresolvedPath
#[derive(ConstParamTy, PartialEq, Eq, Clone, Copy, Debug)]
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

    pub fn color_by_moves(moves: usize) -> Self {
        if moves % 2 == 1 {
            Color::Black
        } else {
            Color::White
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
