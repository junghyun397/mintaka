use crate::board_io::{SYMBOL_BLACK, SYMBOL_WHITE};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use std::marker::ConstParamTy;
use std::ops::Not;
use std::str::FromStr;

//noinspection RsUnresolvedPath
#[derive(ConstParamTy, PartialEq, Eq, Clone, Copy, Debug, Default, Serialize, Deserialize)]
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

impl FromStr for Color {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "black" | "b" => Ok(Color::Black),
            "white" | "w" => Ok(Color::White),
            &_ => Err("unknown color")
        }
    }
}

macro_rules! impl_color_container {
    ($name:ident,$bound:ident) => {
        impl<T: $bound> $name<T> {

            pub const fn new(black: T, white: T) -> Self {
                Self {
                    black,
                    white
                }
            }

            pub const fn access(&self, color: Color) -> &T {
                match color {
                    Color::Black => &self.black,
                    Color::White => &self.white
                }
            }

            pub const fn access_mut(&mut self, color: Color) -> &mut T {
                match color {
                    Color::Black => &mut self.black,
                    Color::White => &mut self.white
                }
            }

            pub const fn get_ref<const C: Color>(&self) -> &T {
                match C {
                    Color::Black => &self.black,
                    Color::White => &self.white
                }
            }

            pub const fn get_reversed_ref<const C: Color>(&self) -> &T {
                match C {
                    Color::Black => &self.white,
                    Color::White => &self.black
                }
            }

            pub const fn get_ref_mut<const C: Color>(&mut self) -> &mut T {
                match C {
                    Color::Black => &mut self.black,
                    Color::White => &mut self.white
                }
            }

            pub const fn get_reversed_ref_mut<const C: Color>(&mut self) -> &mut T {
                match C {
                    Color::Black => &mut self.white,
                    Color::White => &mut self.black
                }
            }

        }
    };
}

macro_rules! impl_color_container_copy {
    ($name:ident) => {
        impl<T: std::marker::Copy> $name<T> {

            pub const fn get<const C: Color>(&self) -> T {
                match C {
                    Color::Black => self.black,
                    Color::White => self.white
                }
            }

            pub const fn get_reversed<const C: Color>(&self) -> T {
                match C {
                    Color::Black => self.white,
                    Color::White => self.black
                }
            }

        }
    }
}

macro_rules! impl_serialize {
    ($name:ident,$bound:ident) => {
        impl<T: $bound + serde::Serialize> serde::Serialize for $name<T> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
                let mut state = serializer.serialize_struct("ColorContainer", 2)?;
                state.serialize_field("black", &self.black)?;
                state.serialize_field("white", &self.white)?;
                state.end()
            }
        }

        impl<'de, T: $bound + serde::Deserialize<'de>> serde::Deserialize<'de> for $name<T> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
                #[derive(Deserialize)]
                struct ColorContainerData<T> {
                    black: T,
                    white: T,
                }

                let data = ColorContainerData::deserialize(deserializer)?;

                Ok(Self::new(data.black, data.white))
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ColorContainer<T: Copy> {
    pub black: T,
    pub white: T,
}

impl_color_container!(ColorContainer, Copy);
impl_color_container_copy!(ColorContainer);
impl_serialize!(ColorContainer, Copy);

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct HeapColorContainer<T> {
    pub black: T,
    pub white: T,
}

impl_color_container!(HeapColorContainer, Clone);
impl_serialize!(HeapColorContainer, Clone);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(align(64))]
pub struct AlignedColorContainer<T: Copy> {
    pub black: T,
    pub white: T,
}

impl_color_container!(AlignedColorContainer, Copy);
impl_color_container_copy!(AlignedColorContainer);
impl_serialize!(AlignedColorContainer, Copy);
