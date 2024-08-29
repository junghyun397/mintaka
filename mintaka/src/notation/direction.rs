use std::marker::ConstParamTy;

#[derive(ConstParamTy, Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum Direction {
    Horizontal = 0,
    Vertical = 8,
    Ascending = 16,
    Descending = 24
}
