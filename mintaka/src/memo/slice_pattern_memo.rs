use crate::slice_pattern::SlicePattern;

pub trait SlicePatternMemo {

    fn probe_or_put_mut<F>(&mut self, raw_slice: u64, produce: F) -> SlicePattern
    where F : FnOnce() -> SlicePattern;

}
