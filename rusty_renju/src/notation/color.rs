use std::marker::ConstParamTy;
use std::ops::Not;

//noinspection RsUnresolvedPath
#[derive(ConstParamTy, PartialEq, Eq, Clone, Copy, Debug, Default)]
#[repr(u8)]
pub enum Color {
    #[default] Black,
    White
}

impl Color {

    pub const fn reversed(&self) -> Color {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black
        }
    }

    pub const fn player_color_from_moves(moves: usize) -> Self {
        if moves % 2 == 0 {
            Color::Black
        } else {
            Color::White
        }
    }

    pub fn player_color_from_each_moves<T: Ord>(black_moves: T, white_moves: T) -> Self {
        if black_moves > white_moves {
            Color::White
        } else {
            Color::Black
        }
    }

}

impl Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ColorContainer<T: Copy> {
    pub black: T,
    pub white: T,
}

impl<T: Copy + Default> Default for ColorContainer<T> {

    fn default() -> Self {
        Self {
            black: T::default(),
            white: T::default()
        }
    }

}

impl<T: Copy> ColorContainer<T> {

    pub fn access(&self, color: Color) -> &T {
        match color {
            Color::Black => &self.black,
            Color::White => &self.white
        }
    }

    pub fn player_unit<const C: Color>(&self) -> &T {
        match C {
            Color::Black => &self.black,
            Color::White => &self.white
        }
    }

    pub fn opponent_unit<const C: Color>(&self) -> &T {
        match C {
            Color::Black => &self.white,
            Color::White => &self.black
        }
    }

    pub fn player_unit_mut<const C: Color>(&mut self) -> &mut T {
        match C {
            Color::Black => &mut self.black,
            Color::White => &mut self.white
        }
    }

    pub fn opponent_unit_mut<const C: Color>(&mut self) -> &mut T {
        match C {
            Color::Black => &mut self.white,
            Color::White => &mut self.black
        }
    }

}
