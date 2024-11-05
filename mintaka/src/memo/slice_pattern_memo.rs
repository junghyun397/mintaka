use crate::slice_pattern::SlicePattern;

pub trait SlicePatternMemo {

    fn prefetch_memo(&self, packed_slice: u64);

    fn probe_or_put_mut<F>(&mut self, packed_slice: u64, produce: F) -> SlicePattern
    where F : FnOnce() -> SlicePattern;

}
