pub trait Empty {
    fn empty() -> Self;
}

impl Empty for u16 {
    fn empty() -> Self {
        0
    }
}
