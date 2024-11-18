use std::marker::ConstParamTy;

//noinspection RsUnresolvedPath
#[derive(ConstParamTy, PartialEq, Eq, Copy, Clone, Debug)]
#[repr(u8)]
pub enum Direction {
    Horizontal = 0,
    Vertical = 1,
    Ascending = 2,
    Descending = 3
}
