#[derive(std::marker::ConstParamTy, PartialEq, Eq, Copy, Clone, Debug)]
#[repr(u8)]
pub enum Direction {
    Horizontal = 0,
    Vertical = 1,
    Ascending = 2,
    Descending = 3
}

impl Direction {

    pub fn from_pattern_position(position: u32) -> Self {
        unsafe { std::mem::transmute::<u8, Self>((position / 8) as u8) }
    }

}
