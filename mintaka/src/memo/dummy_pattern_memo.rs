use crate::memo::slice_pattern_memo::SlicePatternMemo;
use crate::slice_pattern::SlicePattern;

pub struct DummySlicePatternMemo;

impl SlicePatternMemo for DummySlicePatternMemo {

    fn prefetch_memo(&self, _packed_slice: u64) {}

    fn probe_or_put_mut<F>(&mut self, _raw_slice: u64, produce: F) -> SlicePattern
    where F: FnOnce() -> SlicePattern
    {
        produce()
    }

}
