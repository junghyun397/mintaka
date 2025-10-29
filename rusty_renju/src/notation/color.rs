use crate::board_io::{SYMBOL_BLACK, SYMBOL_WHITE};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::Not;
use std::ops::{Index, IndexMut};
use std::str::FromStr;

#[derive(std::marker::ConstParamTy, PartialEq, Eq, Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum Color {
    #[default] Black = 0,
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
        write!(f, "{:?}", self)
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
    (
        $(#[$struct_attr:meta])*
        $name:ident
    ) => {
        $(#[$struct_attr])*
        pub struct $name<T>(pub [T; 2]);

        impl<T> $name<T> {
            #[inline]
            pub const fn new(black: T, white: T) -> Self {
                Self([black, white])
            }

            #[inline]
            pub fn access_pair(&self, color: Color) -> (&T, &T) {
                (&self.0[color as usize], &self.0[color.reversed() as usize])
            }

            #[inline]
            pub const fn get_ref<const C: Color>(&self) -> &T {
                match C {
                    Color::Black => &self.0[0],
                    Color::White => &self.0[1],
                }
            }

            #[inline]
            pub const fn get_reversed_ref<const C: Color>(&self) -> &T {
                match C {
                    Color::Black => &self.0[1],
                    Color::White => &self.0[0],
                }
            }

            #[inline]
            pub const fn get_ref_mut<const C: Color>(&mut self) -> &mut T {
                match C {
                    Color::Black => &mut self.0[0],
                    Color::White => &mut self.0[1],
                }
            }

            #[inline]
            pub const fn get_reversed_ref_mut<const C: Color>(&mut self) -> &mut T {
                match C {
                    Color::Black => &mut self.0[1],
                    Color::White => &mut self.0[0],
                }
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

        impl<T: Copy> $name<T> {

            #[inline]
            pub const fn get<const C: Color>(&self) -> T {
                match C {
                    Color::Black => self.0[0],
                    Color::White => self.0[1],
                }
            }

            #[inline]
            pub const fn get_reversed<const C: Color>(&self) -> T {
                match C {
                    Color::Black => self.0[1],
                    Color::White => self.0[0],
                }
            }
        }
    };
}

impl_color_container!(
    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    ColorContainer
);

impl<T: Copy> Clone for ColorContainer<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Copy> Copy for ColorContainer<T> {}

impl_color_container!(
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
    #[repr(align(64))]
    AlignedColorContainer
);
