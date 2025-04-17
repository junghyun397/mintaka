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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ColorContainer<T: Copy> {
    pub black: T,
    pub white: T,
}

impl_color_container!(ColorContainer, Copy);
impl_color_container_copy!(ColorContainer);

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct HeapColorContainer<T> {
    pub black: T,
    pub white: T,
}

impl_color_container!(HeapColorContainer, Clone);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(align(64))]
pub struct AlignedColorContainer<T: Copy> {
    pub black: T,
    pub white: T,
}

impl_color_container!(AlignedColorContainer, Copy);
impl_color_container_copy!(AlignedColorContainer);
