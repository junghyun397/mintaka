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
