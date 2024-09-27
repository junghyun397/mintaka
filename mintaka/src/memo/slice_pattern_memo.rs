use crate::slice_pattern::SlicePattern;

pub trait SlicePatternMemo {

    fn probe_or_put_mut<F>(&mut self, key: u32, produce: F) -> &SlicePattern
    where F : FnOnce() -> SlicePattern;

}
