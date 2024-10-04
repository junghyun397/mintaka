use crate::memo::slice_pattern_memo::SlicePatternMemo;
use crate::slice::SliceKey;
use crate::slice_pattern::SlicePattern;

pub struct MemoryLeakSlicePatternMemo;

impl SlicePatternMemo for MemoryLeakSlicePatternMemo {

    fn probe_or_put_mut<F>(&mut self, _key: SliceKey, produce: F) -> &SlicePattern
    where F: FnOnce() -> SlicePattern
    {
        Box::leak(Box::new(produce()))
    }

}
