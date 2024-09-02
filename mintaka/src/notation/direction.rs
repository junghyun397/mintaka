use std::marker::ConstParamTy;

//noinspection RsUnresolvedPath
#[derive(ConstParamTy, PartialEq, Eq, Copy, Clone, Debug)]
#[repr(u8)]
pub enum Direction {
    Horizontal = 0,
    Vertical = 8,
    Ascending = 16,
    Descending = 24
}
