use std::fmt::Debug;
use std::ops::{Index, IndexMut};
use crate::utils::empty::Empty;

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

#[repr(transparent)]
pub struct DirectionContainer<T>(pub [T; 4]);

impl<T> DirectionContainer<T> {
    pub const fn new(horizontal: T, vertical: T, ascending: T, descending: T) -> Self {
        Self([horizontal, vertical, ascending, descending])
    }

    pub fn iter(&self) -> impl Iterator<Item = (Direction, &T)> {
        [Direction::Horizontal, Direction::Vertical, Direction::Ascending, Direction::Descending]
            .map(|direction| (direction, &self[direction]))
            .into_iter()
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

impl<T: Empty> Empty for DirectionContainer<T> {
    fn empty() -> Self {
        Self::new(T::empty(), T::empty(), T::empty(), T::empty())
    }
}

impl <T: PartialEq> PartialEq for DirectionContainer<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl <T: Eq> Eq for DirectionContainer<T> { }

impl <T: Copy> Copy for DirectionContainer<T> {}

impl <T: Clone> Clone for DirectionContainer<T> {
    fn clone(&self) -> Self {
        Self::new(self.0[0].clone(), self.0[1].clone(), self.0[2].clone(), self.0[3].clone())
    }
}

impl <T: Default> Default for DirectionContainer<T> {
    fn default() -> Self {
        Self::new(T::default(), T::default(), T::default(), T::default())
    }
}

impl <T: Debug> Debug for DirectionContainer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DirectionContainer")
            .field("horizontal", &self.0[0])
            .field("vertical", &self.0[1])
            .field("ascending", &self.0[2])
            .field("descending", &self.0[3])
            .finish()
    }
}
