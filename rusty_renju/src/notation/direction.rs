use std::ops::{Index, IndexMut};

#[derive(std::marker::ConstParamTy, PartialEq, Eq, Copy, Clone, Debug)]
#[repr(u8)]
pub enum Direction {
    Horizontal = 0,
    Vertical = 1,
    Ascending = 2,
    Descending = 3
}

impl From<u8> for Direction {
    fn from(value: u8) -> Self {
        debug_assert!(value < 4);
        unsafe { std::mem::transmute(value) }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct DirectionContainer<T>(pub [T; 4]);

impl<T> DirectionContainer<T> {
    pub const fn new(horizontal: T, vertical: T, ascending: T, descending: T) -> Self {
        Self([horizontal, vertical, ascending, descending])
    }
}

impl<T> Index<Direction> for DirectionContainer<T> {
    type Output = T;

    fn index(&self, index: Direction) -> &T {
        &self.0[index as usize]
    }
}

impl<T> IndexMut<Direction> for DirectionContainer<T> {
    fn index_mut(&mut self, index: Direction) -> &mut T {
        &mut self.0[index as usize]
    }
}

impl<T: crate::utils::empty::Empty> crate::utils::empty::Empty for DirectionContainer<T> {
    fn empty() -> Self {
        Self::new(T::empty(), T::empty(), T::empty(), T::empty())
    }
}
