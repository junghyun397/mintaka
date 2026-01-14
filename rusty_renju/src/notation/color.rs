use crate::board_io::{SYMBOL_BLACK, SYMBOL_WHITE};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::Not;
use std::ops::{Index, IndexMut};
use std::str::FromStr;
use typeshare::typeshare;

#[typeshare(serialized_as = "ColorSchema")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(std::marker::ConstParamTy, PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    White = 1,
}

impl Color {

    pub const fn reversed(&self) -> Color {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black
        }
    }

    pub const fn player_color_from_moves(moves: usize) -> Self {
        if moves.is_multiple_of(2) {
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


impl From<Color> for char {
    fn from(value: Color) -> Self {
        match value {
            Color::Black => SYMBOL_BLACK,
            Color::White => SYMBOL_WHITE
        }
    }
}

impl From<Color> for u8 {
    fn from(value: Color) -> Self {
        value as u8
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug)]
pub struct UnknownColorError;

impl Display for UnknownColorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown color")
    }
}

impl std::error::Error for UnknownColorError {}

impl FromStr for Color {
    type Err = UnknownColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "black" | "b" => Ok(Color::Black),
            "white" | "w" => Ok(Color::White),
            &_ => Err(UnknownColorError)
        }
    }
}

macro_rules! impl_color_container {
    ($name:ident) => {
        impl<T> $name<T> {
            #[inline]
            pub const fn new(black: T, white: T) -> Self {
                Self([black, white])
            }

            #[inline]
            pub fn access_pair(&self, color: Color) -> (&T, &T) {
                (&self.0[color as usize], &self.0[color.reversed() as usize])
            }

            pub fn iter(&self) -> std::slice::Iter<'_, T> {
                self.0.iter()
            }

            pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
                self.0.iter_mut()
            }
        }

        impl<T> Index<Color> for $name<T> {
            type Output = T;

            #[inline]
            fn index(&self, index: Color) -> &Self::Output {
                &self.0[index as usize]
            }
        }

        impl<T> IndexMut<Color> for $name<T> {
            #[inline]
            fn index_mut(&mut self, index: Color) -> &mut Self::Output {
                &mut self.0[index as usize]
            }
        }
    };
}

#[typeshare::typeshare]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Eq)]
pub struct ColorContainer<T>(pub [T; 2]);

impl_color_container!(ColorContainer);

impl<T: Copy> Clone for ColorContainer<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Copy> Copy for ColorContainer<T> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(align(64))]
pub struct AlignedColorContainer<T>(pub [T; 2]);

impl_color_container!(AlignedColorContainer);

impl<T> From<AlignedColorContainer<T>> for ColorContainer<T> {
    fn from(value: AlignedColorContainer<T>) -> Self {
        Self(value.0)
    }
}
