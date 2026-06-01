use crate::board_io::{SYMBOL_BLACK, SYMBOL_WHITE};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Formatter;
use std::ops::Not;
use std::str::FromStr;
#[cfg(feature = "typeshare")]
use typeshare::typeshare;

#[cfg_attr(feature = "typeshare", typeshare(serialized_as = "ColorSchema"))]
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

impl TryFrom<u8> for Color {
    type Error = UnknownColorError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        const BLACK: u8 = Color::Black as u8;
        const WHITE: u8 = Color::White as u8;

        match value {
            BLACK => Ok(Color::Black),
            WHITE => Ok(Color::White),
            _ => Err(UnknownColorError)
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug)]
pub struct UnknownColorError;

impl std::fmt::Display for UnknownColorError {
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

            pub fn iter(&self) -> impl Iterator<Item = (Color, &T)> {
                [Color::Black, Color::White]
                    .map(|color| (color, &self[color]))
                    .into_iter()
            }
        }

        impl<T> std::ops::Index<Color> for $name<T> {
            type Output = T;

            #[inline]
            fn index(&self, index: Color) -> &Self::Output {
                &self.0[index as usize]
            }
        }

        impl<T> std::ops::IndexMut<Color> for $name<T> {
            #[inline]
            fn index_mut(&mut self, index: Color) -> &mut Self::Output {
                &mut self.0[index as usize]
            }
        }

        impl<T: crate::utils::empty::Empty> crate::utils::empty::Empty for $name<T> {
            fn empty() -> Self {
                Self::new(T::empty(), T::empty())
            }
        }

        impl <T: PartialEq> PartialEq for $name<T> {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl <T: Eq> Eq for $name<T> { }

        impl <T: Copy> Copy for $name<T> {}

        impl <T: Clone> Clone for $name<T> {
            fn clone(&self) -> Self {
                Self::new(self[Color::Black].clone(), self[Color::White].clone())
            }
        }

        impl <T: std::fmt::Debug> std::fmt::Debug for $name<T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("ColorContainer")
                    .field("black", &self.0[0])
                    .field("white", &self.0[1])
                    .finish()
            }
        }

        impl<T: Default> Default for $name<T> {
            fn default() -> Self {
                Self::new(T::default(), T::default())
            }
        }
    };
}

#[cfg_attr(feature = "typeshare", typeshare)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ColorContainer<T>(pub [T; 2]);
impl_color_container!(ColorContainer);

#[repr(align(64))]
pub struct AlignedColorContainer<T>(pub [T; 2]);
impl_color_container!(AlignedColorContainer);
